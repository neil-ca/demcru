use std::{collections::HashMap, net::TcpListener};

use crate::{configuration::Config, contacts::Contacts};
use actix_files::Files;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::Error;
use actix_web::{
    cookie::{time::Duration, Cookie, Key},
    dev::Server,
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use handlebars::Handlebars;
use serde_json::json;
use sqlx::{query, PgPool};
use uuid::Uuid;
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub async fn index(hb: Data<Handlebars<'static>>, req: HttpRequest) -> HttpResponse {
    let user_uuid = match req.cookie("user_uuid") {
        Some(_) => true,
        None => false,
    };
    let content = if user_uuid {
        hb.render("index", &json!({"cookie": true})).unwrap()
    } else {
        hb.render("index", &json!({"cookie": false})).unwrap()
    };
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content)
}

pub async fn like(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    let user_uuid = match req.cookie("user_uuid") {
        Some(c) => {
            // Delete like from db
            let uuid_str = c.value();
            let id = match Uuid::parse_str(uuid_str) {
                Ok(parsed) => parsed,
                Err(_) => {
                    return Ok(HttpResponse::InternalServerError().body("Failed to parse Uuid"))
                }
            };
            let _ = query!("DELETE FROM likes WHERE id = $1", id)
                .execute(pool.get_ref())
                .await;

            // Delete cookie
            let expiration_cookie = Cookie::build("user_uuid", "")
                .max_age(Duration::ZERO)
                .path("/")
                .finish();
            HttpResponse::Ok()
                .cookie(expiration_cookie)
                .content_type("text/html; charset=utf-8")
                .body(
                    r#"<img src="/images/dislike.svg" class="w-6 h-6 hover:w-7 
                        hover:h-7 m-2 animate-pulse"
                        hx-post="/like" hx-swap="outerHTML"/>
                        "#,
                )
        }
        None => {
            let count = query!("SELECT COUNT(id) FROM likes")
                .fetch_one(pool.get_ref())
                .await;
            match count {
                Ok(likes) => {
                    let current: i64 = likes.count.unwrap_or(0);
                    let new = current + 1;
                    let new_uuid = Uuid::new_v4();
                    let _ = query!(
                        "INSERT INTO likes (id, counter) VALUES ($1, $2)",
                        new_uuid,
                        new
                    )
                    .execute(pool.get_ref())
                    .await;
                    HttpResponse::Ok()
                        .cookie(
                            Cookie::build("user_uuid", new_uuid.to_string())
                                .http_only(true)
                                .path("/")
                                .finish(),
                        )
                        .content_type("text/html; charset=utf-8")
                        .body(
                            r#"<img src="/images/heart.svg" class="w-6 h-6 hover:w-7 
                    hover:h-7 m-2 animate-pulse"
            hx-post="/like" hx-swap="outerHTML"/>
            "#,
                        )
                }
                Err(_) => return Ok(HttpResponse::InternalServerError().body("Failed to parse Uuid"))
            }
        }
    };
    Ok(user_uuid)
}

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

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // Wrap the connections in a smart poiner
    let config = Config::new();
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".hbs", "./templates")
        .unwrap();
    let secret_key = Key::generate();
    let conn = Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_secure(false)
                    .build(),
            )
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
