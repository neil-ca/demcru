use serde_json::json;
use handlebars::Handlebars;
use actix_web::{web, Responder, HttpResponse};

use crate::configuration::Config;

pub async fn blog(hb: web::Data<Handlebars<'_>>, config: web::Data<Config>) -> impl Responder {
    let default = config.default.clone();
    current(hb, config, default)
}

pub async fn detail(
    hb: web::Data<Handlebars<'_>>,
    config: web::Data<Config>,
    path: web::Path<String>,
) -> impl Responder {
    current(hb, config, path.into_inner())
}

pub fn current(
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

pub async fn content(
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
