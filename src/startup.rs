use std::{net::TcpListener, collections::HashMap};

use actix_files::Files;
use actix_web::{App, HttpServer, dev::Server, web::{self, Data}, Responder, HttpResponse};
use sqlx::PgPool;
use handlebars::Handlebars;
use serde_json::json;
use crate::{configuration::Config, contacts::Contacts};

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub async fn index(hb: Data<Handlebars<'static>>) -> impl Responder {
    let html = hb.render("index", &json!({})).unwrap();
    HttpResponse::Ok().body(html)
}
pub async fn contacts(
    hb: web::Data<Handlebars<'static>>,
    pool: web::Data<PgPool>) 
-> impl Responder {
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

async fn blog(hb: web::Data<Handlebars<'_>>, config: web::Data<Config>) -> impl Responder {
    let default = config.default.clone();
    current(hb, config, default)
}

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
    let body = hb.render("blog", &data).unwrap();
    HttpResponse::Ok().body(body)
}

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

    let conn = Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(conn.clone())
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(handlebars.clone()))
            .route("/", web::get().to(index))
            .route("/health-check", web::get().to(health_check))
            .route("/contacts", web::get().to(contacts))
            .route("/blog/{current}", web::get().to(detail))
            .route("/blog", web::get().to(blog))
            .route("/blog/content/{slug}", web::get().to(content))
            // .service(detail)
            // .service(content)
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

