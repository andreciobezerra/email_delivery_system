use std::net::TcpListener;

use email_delivery_system::{
    configs::{get_configs, DatabaseSettings},
    startup::run,
};
use sqlx::{types::Uuid, Connection, Executor, PgConnection, PgPool};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect(&config.connection_string_without_database_name())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");
    // Migrate database
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind a random port.");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");
    let mut configs = get_configs().expect("Failed to read configuration");

    configs.database.database_name = Uuid::new_v4().to_string();

    let db_pool = configure_database(&configs.database).await;

    let server = run(listener, db_pool.clone()).expect("Failed to bind address");

    tokio::spawn(server);

    TestApp { address, db_pool }
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_201_for_valid_form_data() {
    let app = spawn_app().await;
    let configuration = get_configs().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let client = reqwest::Client::new();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to Connect to Postgres");

    let payload = "name=xablau%20silva&email=xablauzin%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(201, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "xablauzin@gmail.com");
    assert_eq!(saved.name, "xablau silva");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=xablau%20silva", "missing the email"),
        ("email=xablauzin%20gmail.com", "missing the name"),
        ("", "Missing both name and email"),
    ];

    for (invalid_payload, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlenconded")
            .body(invalid_payload)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
