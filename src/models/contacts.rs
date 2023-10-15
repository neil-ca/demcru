use sqlx::types::Uuid;
use sqlx::types::chrono::Utc;
use chrono::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct Contacts {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<i16>,
    pub email: String,
    pub created_at: DateTime<Utc>,
}
