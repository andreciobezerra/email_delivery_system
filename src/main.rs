use std::net::TcpListener;

use email_delivery_system::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:8000").expect("Failed to bind the port 8000.");
    run(listener)?.await
}
