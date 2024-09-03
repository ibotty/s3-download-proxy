use std::{fmt::Display, time::Duration};

use aws_sdk_s3::{
    error::SdkError,
    operation::head_object::{HeadObjectError, HeadObjectOutput},
    presigning::{PresignedRequest, PresigningConfig},
    Client,
};

pub(crate) async fn stat_file(
    s3_client: &Client,
    bucket: impl Into<String>,
    key: impl Into<String>,
) -> Result<HeadObjectOutput, SdkError<HeadObjectError>> {
    s3_client.head_object().bucket(bucket).key(key).send().await
}

pub(crate) async fn presign_get(
    s3_client: &Client,
    bucket: impl Into<String>,
    key: impl Into<String>,
    presigned_ttl: Duration,
    preferred_name: impl Display,
) -> anyhow::Result<PresignedRequest> {
    let presigning_config = PresigningConfig::builder()
        .expires_in(presigned_ttl)
        .build()?;

    let content_desposition = format!(r#"attachment; filename ="{}""#, &preferred_name);

    let result = s3_client
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

    use aws_sdk_s3::error::SdkError;

    #[tokio::test]
    async fn test_url_stat_existing() {
        let endpoint_url = env::var("AWS_ENDPOINT_URL").unwrap();
        let s3_config = crate::s3_config_from_env()
            .await
            .to_builder()
            .endpoint_url(endpoint_url)
            .build();

        let bucket = env::var("S3_BUCKET").unwrap();
        let key = "test-key";

        let s3_client = aws_sdk_s3::Client::from_conf(s3_config);
        let resp = crate::s3::stat_file(&s3_client, bucket, key).await;

        assert!(matches!(resp, Ok(_)))
    }

    #[tokio::test]
    async fn test_url_stat_not_existing() {
        let endpoint_url = env::var("AWS_ENDPOINT_URL").unwrap();
        let s3_config = crate::s3_config_from_env()
            .await
            .to_builder()
            .endpoint_url(endpoint_url)
            .build();

        let bucket = env::var("S3_BUCKET").unwrap();
        let key = "test-key-does-not-exist";

        let s3_client = aws_sdk_s3::Client::from_conf(s3_config);
        let resp = crate::s3::stat_file(&s3_client, bucket, key).await;

        match resp {
            Err(SdkError::ServiceError(err)) => {
                if !err.err().is_not_found() {
                    panic!("err: {:?}", err)
                }
            }
            _ => panic!("wrong response: {:?}", resp),
        }
    }

    #[tokio::test]
    async fn test_presign_url() {
        let endpoint_url = env::var("AWS_ENDPOINT_URL").unwrap();
        let s3_config = crate::s3_config_from_env()
            .await
            .to_builder()
            .endpoint_url(endpoint_url)
            .build();

        let bucket = env::var("S3_BUCKET").unwrap();
        let key = "test-key";
        let preferred_name = "test.txt";
        let presigned_ttl = Duration::from_secs(5);

        let s3_client = aws_sdk_s3::Client::from_conf(s3_config);
        let req = crate::s3::presign_get(&s3_client, bucket, key, presigned_ttl, preferred_name)
            .await
            .unwrap();
        let url = req.uri();

        let text = reqwest::get(url).await.unwrap().text().await.unwrap();
        assert_eq!(text, "TEST\n");
    }
}
