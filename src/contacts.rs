use serde::{Deserialize, Serialize};
#[derive(Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct Contacts {
    pub email: String,
}
