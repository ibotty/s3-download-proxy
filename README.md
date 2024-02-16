# A Proxy that redirects to presigned S3 URLs

When a request in the form of `https://downloads.example.com/some-long-secret/file.txt`
it will check in a PostgreSQL database whether the secret exists and get the bucket and key in the bucket.
Then it will presign a GET request to this resource and 307 redirect to its URL.


## Database Setup

See `samples/*.sql` for a possible database config.


## Neccessary Config for custom s3

 * `AWS_ACCESS_KEY_ID`,
 * `AWS_SECRET_ACCESS_KEY`,
 * `AWS_REGION=us-east-1`, required if `aws_endpoint_url` is used ([See bug report](https://github.com/smithy-lang/smithy-rs/issues/3403))
