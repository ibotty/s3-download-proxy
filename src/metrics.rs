use foundations::telemetry::metrics::metrics;

#[metrics]
pub(crate) mod server {
    // Number of active connections
    //pub fn active_connections(endpoint_name: &Arc<String>) -> Gauge

    // Number of active connections
    //pub fn failed_connections(endpoint_name: &Arc<String>) -> Gauge
}
