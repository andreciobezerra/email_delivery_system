use std::net::TcpListener;

use email_delivery_system::configs::get_configs;
use email_delivery_system::startup::run;
use email_delivery_system::telemetry::{get_subscriber, init_subscriber};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("xablau".to_string(), "info".to_string());

    init_subscriber(subscriber);

    let configs = get_configs().expect("Failed to read configs");
    let address = format!("127.0.0.1:{}", configs.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind the port 8000.");
    let db_connection_pool = PgPool::connect(&configs.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    run(listener, db_connection_pool)?.await
}
