use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    DatabaseError(String),
    NotFound(String),
    BadRequest(String),
    InternalError(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    message: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let response = ErrorResponse {
            success: false,
            message: self.to_string(),
        };
        match self {
            AppError::DatabaseError(_) => HttpResponse::InternalServerError().json(response),
            AppError::NotFound(_) => HttpResponse::NotFound().json(response),
            AppError::BadRequest(_) => HttpResponse::BadRequest().json(response),
            AppError::InternalError(_) => HttpResponse::InternalServerError().json(response),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::DatabaseError(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::InternalError(e.to_string())
    }
}
