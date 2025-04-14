use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    InternalError(String),
    ValidationError(String),
    NotFoundError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::NotFoundError(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::InternalError(msg) => {
                HttpResponse::InternalServerError().json(json_error("internal_error", msg))
            }
            AppError::ValidationError(msg) => {
                HttpResponse::BadRequest().json(json_error("validation_error", msg))
            }
            AppError::NotFoundError(msg) => {
                HttpResponse::NotFound().json(json_error("not_found", msg))
            }
        }
    }
}

fn json_error(error_code: &str, message: &str) -> serde_json::Value {
    serde_json::json!({
        "error": {
            "code": error_code,
            "message": message
        }
    })
}

pub type AppResult<T> = Result<T, AppError>;
