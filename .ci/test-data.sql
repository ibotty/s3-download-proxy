ALTER USER "bot-download-proxy" PASSWORD 'pass';
ALTER TABLE download_proxy_files ALTER COLUMN aws_endpoint_url SET DEFAULT '${AWS_ENDPOINT_URL}';
ALTER TABLE download_proxy_files ALTER COLUMN aws_region SET DEFAULT '${AWS_REGION}';
ALTER TABLE download_proxy_files ALTER COLUMN s3_bucket SET DEFAULT '${S3_BUCKET}';
ALTER TABLE download_proxy_files ALTER COLUMN aws_s3_force_path_style SET DEFAULT TRUE;

SELECT create_download_proxy_link('${S3_BUCKET}', 'test-key', 'test.txt');
