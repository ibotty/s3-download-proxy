CREATE TABLE IF NOT EXISTS download_proxy_files(
    uuid_download_proxy_files UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    secret CHARACTER(64) NOT NULL UNIQUE DEFAULT REPLACE('' || gen_random_uuid() || gen_random_uuid(), '-', '' ),
    s3_bucket TEXT NOT NULL,
    bucket_key TEXT NOT NULL,
    aws_endpoint_url TEXT,
    aws_region TEXT,
    aws_s3_force_path_style BOOL
);

CREATE OR REPLACE FUNCTION create_download_proxy_link(s3_bucket TEXT, bucket_key TEXT, preferred_name TEXT) RETURNS TEXT
AS $$
    INSERT INTO download_proxy_files(s3_bucket, bucket_key)
    VALUES ($1, $2)
    RETURNING 'https://downloads.example.com/' || secret || '/' || $2 AS url
$$ LANGUAGE SQL
STRICT;

CREATE OR REPLACE FUNCTION download_proxy_file_info(secret CHARACTER(64))
RETURNS TABLE(
    id UUID,
    s3_bucket TEXT,
    bucket_key TEXT,
    aws_endpoint_url TEXT,
    aws_region TEXT,
    aws_s3_force_path_style BOOL
) AS $$
    SELECT uuid_download_proxy_files AS "id", s3_bucket, bucket_key, aws_endpoint_url, aws_region, aws_s3_force_path_style
    FROM download_proxy_files
    WHERE secret = $1
$$ LANGUAGE SQL
SET search_path to public
STRICT
STABLE
SECURITY DEFINER
ROWS 1;
REVOKE ALL ON FUNCTION download_proxy_file_info FROM PUBLIC;

CREATE TABLE IF NOT EXISTS download_proxy_access_log(
    id_download_proxy_access_log UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    uuid_download_proxy_files UUID NOT NULL REFERENCES download_proxy_files(uuid_download_proxy_files),
    access_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    access_data JSONB NOT NULL DEFAULT '{}'
);
