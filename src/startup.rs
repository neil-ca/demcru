use std::{
    collections::HashMap,
    net::TcpListener,
    sync::{atomic::AtomicUsize, Arc},
};

use crate::{
    configuration::Config,
    models::contacts::Contacts,
    routes::{
        blog, chat, chat_route, content, detail, get_count, health_check, index, like, ChatServer,
    },
};
use actix::Actor;
use actix_files::Files;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
    dev::Server,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use anyhow::Result;
use handlebars::Handlebars;
use sqlx::PgPool;

pub async fn contacts(
    hb: web::Data<Handlebars<'static>>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let body = sqlx::query_as!(Contacts, "SELECT * FROM contacts")
        .fetch_all(pool.get_ref())
        .await;
    match body {
        Ok(contacts) => {
            // let data = json!({
            //     "contacts": contacts
            // });
            // println!("{:?}", data);
            let mut context = HashMap::new();
            context.insert("contacts".to_string(), &contacts);
            let html = hb.render("contact", &context).unwrap();
            HttpResponse::Ok().body(html)
        }
        Err(e) => {
            print!("Failed: {}", e);
            HttpResponse::InternalServerError().body(format!("Error: {}", e))
        }
    }
}

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // Wrap the connections in a smart poiner
    let config = Config::new();
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".hbs", "./templates")
        .unwrap();
    let secret_key = Key::generate();
    let conn = Data::new(db_pool);
    // ws
    let app_state = Arc::new(AtomicUsize::new(0));
    let chat_server = ChatServer::new(app_state.clone()).start();
    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_secure(false)
                    .build(),
            )
            .app_data(web::Data::from(app_state.clone()))
            .app_data(Data::new(chat_server.clone()))
            .app_data(conn.clone())
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(handlebars.clone()))
            .route("/", web::get().to(index))
            .route("/health-check", web::get().to(health_check))
            .route("/like", web::post().to(like))
            .route("/contacts", web::get().to(contacts))
            .route("/blog/{current}", web::get().to(detail))
            .route("/blog", web::get().to(blog))
            .route("/blog/content/{slug}", web::get().to(content))
            .route("/chat", web::get().to(chat))
            .route("/ws", web::get().to(chat_route))
            .route("/count", web::get().to(get_count))
            .service(
                Files::new("/", "./static")
                    .prefer_utf8(true)
                    .use_last_modified(true),
            )
    })
    .listen(listener)?
    .run();
    Ok(server)
}
