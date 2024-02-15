use std::{sync::Arc, time::Duration};

use sqlx::PgPool;

pub(crate) type State = Arc<ServerState>;

impl ServerState {
    pub(crate) fn new(
        pg_pool: PgPool,
        s3_config: aws_sdk_s3::Config,
        presigned_ttl: Duration,
    ) -> State {
        Arc::new(ServerState {
            pg_pool,
            s3_config,
            presigned_ttl,
        })
    }
}

#[derive(Clone)]
pub(crate) struct ServerState {
    pub(crate) pg_pool: PgPool,
    pub(crate) s3_config: aws_sdk_s3::Config,
    pub(crate) presigned_ttl: Duration,
}
