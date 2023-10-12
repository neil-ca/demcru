use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum CustomError {
    ParsingError,
    DatabaseError(sqlx::Error),
}

impl ResponseError for CustomError {
    fn error_response(&self) -> HttpResponse {
        match self {
            CustomError::ParsingError => {
                HttpResponse::InternalServerError().body("Failed to parse data")
            }
            CustomError::DatabaseError(_) => {
                HttpResponse::InternalServerError().body("Database error")
            }
        }
    }
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Implement the Display trait for your error messages
        match self {
            CustomError::ParsingError => write!(f, "Failed to parse data"),
            CustomError::DatabaseError(err) => write!(f, "Database error: {:?}", err),
        }
    }
}

