use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
#[allow(clippy::enum_variant_names)] // Keep original names for now, or rename as below
pub enum AppError {
    Internal(String),
    Validation(String),
    // NotFoundError(String), // Clippy reported this variant is never constructed
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
            AppError::Validation(msg) => write!(f, "Validation error: {}", msg),
            // AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Internal(msg) => {
                HttpResponse::InternalServerError().json(json_error("internal_error", msg))
            }
            AppError::Validation(msg) => {
                HttpResponse::BadRequest().json(json_error("validation_error", msg))
            } // AppError::NotFound(msg) => {
              //     HttpResponse::NotFound().json(json_error("not_found", msg))
              // }
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
