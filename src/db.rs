use crate::AppError;

use sqlx::types::Uuid;

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
