mod apperror;
mod db;
mod metrics;
mod s3;
mod state;

use std::env;
use std::future::IntoFuture;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use self::apperror::*;
use self::state::*;

use anyhow::Context;
use axum::extract::ConnectInfo;
use axum::routing::get_service;
use axum::{extract::Path, response::Redirect};
use db::DownloadInfo;
use foundations::{
    cli::{Arg, ArgAction, Cli},
    telemetry::{self, log, settings::TelemetrySettings},
    BootstrapResult,
};
use tokio::net::TcpListener;
use tokio::signal::unix;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> BootstrapResult<()> {
    let service_info = foundations::service_info!();
    let cli = Cli::<TelemetrySettings>::new(
        &service_info,
        vec![Arg::new("check")
            .long("check")
            .action(ArgAction::SetTrue)
            .help("Validate config.")],
    )?;

    if cli.arg_matches.get_flag("check") {
        return Ok(());
    }

    let redirect_homepage = env::var("REDIRECT_HOMEPAGE").expect("Env REDIRECT_HOMEPAGE not set.");

    let telemetry_fut = telemetry::init_with_server(&service_info, &cli.settings, vec![])?;
    if let Some(addr) = telemetry_fut.server_addr() {
        log::info!("Telemetry server listening on http://{}", addr);
    }

    let presigned_ttl = env::var("PRESIGNED_TTL")
        .map(|str| u64::from_str(&str).unwrap())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(60));

    let s3_config = s3_config_from_env().await;

    let pg_connect_str = env::var("DATABASE_URL")?;
    let pg_pool = sqlx::PgPool::connect(&pg_connect_str).await?;

    let server_state = ServerState::new(pg_pool, s3_config, presigned_ttl, redirect_homepage);

    let bind_addr = "0.0.0.0:8080";

    let governor_config = Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .context("Cannot construct governor layer config")?,
    );

    let app = axum::Router::new()
        .layer(GovernorLayer {
            config: governor_config,
        })
        .route("/", axum::routing::get(redirect_to_homepage))
        .route("/robots.txt", axum::routing::get(robots_txt))
        .route("/:id/:path", axum::routing::get(get_handler))
        .with_state(server_state)
        .fallback(get_service(serve_statics()));
    let listener = TcpListener::bind(bind_addr).await?;
    let axum_fut = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .into_future();

    log::info!("Server listening on http://{}", bind_addr);

    #[cfg(target_os = "linux")]
    sandbox_syscalls()?;

    //axum_fut.await?;

    tokio::select! {
        r = telemetry_fut => { r? },
        r = axum_fut => { r? },
    }

    Ok(())
}

async fn s3_config_from_env() -> aws_sdk_s3::Config {
    let aws_config = aws_config::from_env().load().await;

    let aws_force_path_style = matches!(
        env::var("AWS_S3_FORCE_PATH_STYLE")
            .unwrap_or("false".to_string())
            .as_str(),
        "true" | "TRUE" | "1"
    );

    aws_sdk_s3::config::Builder::from(&aws_config)
        .force_path_style(aws_force_path_style)
        .build()
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    let terminate = async {
        unix::signal(unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    log::info!("signal received, starting graceful shutdown");
}

async fn s3_config(s3_config: &aws_sdk_s3::Config, info: &DownloadInfo) -> aws_sdk_s3::Config {
    let mut s3_config_builder = s3_config.to_builder();

    if let Some(aws_endpoint_url) = info.aws_endpoint_url.as_ref() {
        s3_config_builder = s3_config_builder.endpoint_url(aws_endpoint_url);
    };

    if let Some(aws_s3_force_path_style) = info.aws_s3_force_path_style {
        s3_config_builder = s3_config_builder.force_path_style(aws_s3_force_path_style)
    };

    if let Some(aws_region) = info.aws_region.as_ref() {
        let region = aws_config::Region::new(aws_region.clone());
        s3_config_builder = s3_config_builder.region(region);
    };

    s3_config_builder.build()
}

#[axum::debug_handler]
async fn redirect_to_homepage(state: axum::extract::State<Arc<ServerState>>) -> Redirect {
    Redirect::permanent(&state.redirect_homepage)
}

async fn robots_txt() -> &'static str {
    " User-agent: * \n Disallow: / \n"
}

#[axum::debug_handler]
async fn get_handler(
    state: axum::extract::State<Arc<ServerState>>,
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
    Path((secret, preferred_name)): Path<(String, String)>,
) -> Result<Redirect, AppError> {
    let info = db::get_download_info(&state.pg_pool, &secret).await?;

    let s3_config = s3_config(&state.s3_config, &info).await;
    let s3_client = aws_sdk_s3::Client::from_conf(s3_config);

    log::debug!("checking for file {:?}", info);
    let _ = s3::stat_file(&s3_client, &info.bucket_key, &info.bucket_key).await?;

    log::debug!("presigning GET for {:?}", info);
    let req = s3::presign_get(
        &s3_client,
        &info.s3_bucket,
        &info.bucket_key,
        state.presigned_ttl,
        preferred_name,
    )
    .await?;

    let client_data = vec![("client_ip".to_string(), format!("{}", client_addr.ip()))];

    db::log_access(&state.pg_pool, info.uuid, client_data.into_iter()).await?;
    Ok(Redirect::temporary(req.uri()))
}

#[cfg(target_os = "linux")]
fn sandbox_syscalls() -> BootstrapResult<()> {
    use foundations::security::{common_syscall_allow_lists::*, *};

    allow_list! {
        static ALLOWED = [
            ..ASYNC,
            ..SERVICE_BASICS,
            ..NET_SOCKET_API
        ]
    }
    enable_syscall_sandboxing(ViolationAction::KillProcess, &ALLOWED)
}

fn serve_statics() -> ServeDir {
    let serve_dir = env::var("STATIC_DIR").unwrap_or("./assets".to_string());
    ServeDir::new(serve_dir)
}
