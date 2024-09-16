use crate::AppError;

use anyhow::Context;
use sqlx::{
    types::{JsonValue, Uuid},
    FromRow, PgPool,
};

#[derive(Debug, FromRow)]
pub(crate) struct DownloadInfo {
    #[sqlx(rename = "id")]
    pub uuid: Uuid,
    pub s3_bucket: String,
    pub bucket_key: String,
    pub aws_endpoint_url: Option<String>,
    pub aws_region: Option<String>,
    pub aws_s3_force_path_style: Option<bool>,
}

pub(crate) async fn get_download_info(
    pool: &PgPool,
    secret: &str,
) -> Result<DownloadInfo, AppError> {
    let result = sqlx::query_as(
        r#"SELECT id, s3_bucket, bucket_key, aws_endpoint_url, aws_region, aws_s3_force_path_style FROM download_proxy_file_info( $1 );"#,
    )
    .bind(secret)
    .fetch_optional(pool)
    .await.context("Cannot fetch download_info")?;
    result.ok_or(AppError::Unauthorized)
}

pub(crate) async fn log_access(
    pool: &PgPool,
    download_id: Uuid,
    access_data: impl IntoIterator<Item = (String, String)>,
) -> Result<(), AppError> {
    let access_data: JsonValue = JsonValue::from_iter(access_data);
    let _ = sqlx::query!(
        r#"INSERT INTO download_proxy_access_log (uuid_download_proxy_files, access_data) VALUES ($1, $2)"#,
        download_id,
        access_data
    )
    .execute(pool)
    .await
    .context("cannot log access to DB")?;
    Ok(())
}
