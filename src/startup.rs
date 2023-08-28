use std::net::TcpListener;

use actix_files::Files;
use actix_web::{App, HttpServer, dev::Server, web::Data};
use sqlx::PgPool;

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
) -> Result<Server, std::io::Error> {
    // Wrap the connections in a smart poiner

    let db_pool = Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .service(Files::new("/images", "static/images/").show_files_listing())
            .service(Files::new("/", "./static/root/").index_file("index.html"))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}

