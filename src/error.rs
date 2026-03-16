use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Unauthorized(String),
    InternalError(String),
    DatabaseError(String),
    TemplateError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal Error: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            AppError::TemplateError(msg) => write!(f, "Template Error: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::TemplateError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}

impl From<uuid::Error> for AppError {
    fn from(_err: uuid::Error) -> Self {
        AppError::NotFound("Invalid ID".to_string())
    }
}

impl From<tera::Error> for AppError {
    fn from(err: tera::Error) -> Self {
        AppError::TemplateError(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::InternalError(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
