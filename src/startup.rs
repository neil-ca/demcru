use std::net::TcpListener;

use actix_files::Files;
use actix_web::{App, HttpServer, dev::Server, web::{self, Data}, get, Responder, HttpResponse};
use sqlx::PgPool;
use handlebars::Handlebars;
use serde_json::json;
use crate::configuration::Config;

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[get("/")]
async fn index(hb: web::Data<Handlebars<'_>>, config: web::Data<Config>) -> impl Responder {
    let default = config.default.clone();
    current(hb, config, default)
}

#[get("/{current}")]
async fn detail(
    hb: web::Data<Handlebars<'_>>, 
    config: web::Data<Config>,
    path: web::Path<String>,
) -> impl Responder {
    current(hb, config, path.into_inner())
}

fn current(
    hb: web::Data<Handlebars>,
    config: web::Data<Config>,
    current: String,
) -> impl Responder {
    let data = json!({
        "title": config.title,
        "description": config.description,
        "posts": config.posts,
        "current": current
    });
    let body = hb.render("index", &data).unwrap();
    HttpResponse::Ok().body(body)
}

#[get("/content/{slug}")]
async fn content(
    config: web::Data<Config>,
    hb: web::Data<Handlebars<'_>>,
    path: web::Path<String>,
) -> impl Responder {
    let slug = path.into_inner();
    let post = config.posts.iter().find(|post| post.slug == slug).unwrap();
    let data = json!({
        "slug": slug,
        "title": post.title,
        "author": post.author,
        "date": post.date,
        "body": post.render(),
    });
    let body = hb.render("content", &data).unwrap();

    HttpResponse::Ok().body(body)
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
) -> Result<Server, std::io::Error> {
    // Wrap the connections in a smart poiner
    let config = Config::new();
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".hbs", "./templates")
        .unwrap();

    let db_pool = Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(db_pool.clone())
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(handlebars.clone()))
            .service(index)
            .service(detail)
            .service(content)
            .service(
                Files::new("/", "./static")
                    .prefer_utf8(true)
                    .use_last_modified(true),
            )
            .route("/health-check", web::get().to(health_check))
    })
    .listen(listener)?
    .run();
    Ok(server)
}

