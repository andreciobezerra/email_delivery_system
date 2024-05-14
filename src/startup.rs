use std::net::TcpListener;

use actix_web::{dev::Server, middleware::Logger, web, App, HttpServer};
use sqlx::PgPool;

use crate::routes::{health_check, subscribe};

pub fn run(listener: TcpListener, pool: PgPool) -> Result<Server, std::io::Error> {
    let db_connection_pool = web::Data::new(pool);

    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .wrap(Logger::default())
            .app_data(db_connection_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
