use actix_web::{
    cookie::{time::Duration, Cookie},
    web::{self, Data},
    HttpRequest, HttpResponse,
};
use handlebars::Handlebars;
use serde_json::json;
use sqlx::query;
use sqlx::PgPool;
use uuid::Uuid;

use crate::utils::CustomError;

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

pub async fn like(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse, CustomError> {
    let user_uuid = match req.cookie("user_uuid") {
        Some(c) => {
            // Delete like from db
            let uuid_str = c.value();
            let id = Uuid::parse_str(uuid_str).map_err(|_| CustomError::ParsingError)?;
            query!("DELETE FROM likes WHERE id = $1", id)
                .execute(pool.get_ref())
                .await
                .map_err(|e| CustomError::DatabaseError(e))?;

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
                    hover:h-7 m-2 cursor-pointer"
                    hx-post="/like" hx-swap="outerHTML"
                    _="on htmx:afterOnLoad put 'Thank you!' into #message2 wait 1s put '' into #message2"
                    />
                    "#,
                )
        }
        None => {
            let count = query!("SELECT COUNT(id) FROM likes")
                .fetch_one(pool.get_ref())
                .await
                .map_err(|e| CustomError::DatabaseError(e))?;

            let current: i64 = count.count.unwrap_or(0);
            let new = current + 1;
            let new_uuid = Uuid::new_v4();
            let _ = query!(
                "INSERT INTO likes (id, counter) VALUES ($1, $2)",
                new_uuid,
                new
            )
            .execute(pool.get_ref())
            .await
            .map_err(|e| CustomError::DatabaseError(e))?;

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
                    hover:h-7 m-2 cursor-pointer"
                    hx-post="/like" hx-swap="outerHTML"
                    _="on htmx:afterOnLoad put 'Thank you!' into #message2 wait 1s put '' into #message2"
           />
            "#,
                )
        }
    };
    Ok(user_uuid)
}
