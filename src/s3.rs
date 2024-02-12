use std::{fmt::Display, time::Duration};

use aws_sdk_s3::presigning::{PresignedRequest, PresigningConfig};

pub(crate) async fn presign_get(
    s3_config: aws_sdk_s3::Config,
    bucket: impl Into<String>,
    key: impl Into<String>,
    presigned_ttl: Duration,
    preferred_name: impl Display,
) -> anyhow::Result<PresignedRequest> {
    let client = aws_sdk_s3::Client::from_conf(s3_config);
    let presigning_config = PresigningConfig::builder()
        .expires_in(presigned_ttl)
        .build()?;

    let content_desposition = format!(r#"attachment; filename ="{}""#, &preferred_name);

    let result = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .response_content_disposition(content_desposition)
        .presigned(presigning_config)
        .await?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::{env, time::Duration};

    #[tokio::test]
    async fn test_presign_url() {
        let s3_config = crate::s3_config_from_env().await;
        let bucket = env::var("S3_BUCKET").unwrap();
        let key = "test-key";
        let presigned_ttl = Duration::from_secs(500);

        let req = crate::s3::presign_get(s3_config, bucket, key, presigned_ttl)
            .await
            .unwrap();
        let url = req.uri();

        let text = reqwest::get(url).await.unwrap().text().await.unwrap();
        assert_eq!(text, "TEST\n");
    }
}
