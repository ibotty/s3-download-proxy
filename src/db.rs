use crate::AppError;

use sqlx::types::{JsonValue, Uuid};

#[derive(sqlx::FromRow)]
pub(crate) struct DownloadInfo {
    #[sqlx(rename = "id")]
    pub uuid: Uuid,
    pub s3_bucket: String,
    pub bucket_key: String,
    pub s3_endpoint_url: String,
}

pub(crate) async fn get_download_info(
    pool: &sqlx::Pool<sqlx::Postgres>,
    secret: &str,
) -> Result<DownloadInfo, AppError> {
    let result = sqlx::query_as(
        r#"SELECT id, s3_bucket, bucket_key, s3_endpoint_url FROM download_proxy_file_info( $1 );"#,
    )
    .bind(secret)
    .fetch_optional(pool)
    .await?;
    result.ok_or(AppError::Unauthorized)
}

pub(crate) async fn log_access(
    pool: &sqlx::Pool<sqlx::Postgres>,
    download_id: Uuid,
    access_data: impl IntoIterator<Item = (String, String)>,
) -> Result<(), AppError> {
    let access_data: JsonValue = JsonValue::from_iter(access_data);
    let _ = sqlx::query!(r#"INSERT INTO download_proxy_access_log (uuid_download_proxy_files, access_data) VALUES ($1, $2)"#, download_id, access_data).execute(pool).await?;
    Ok(())
}
