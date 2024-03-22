use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub enum Error {
    // We'll convert errors from sqlx::Error into an HTTP status code and message.
    Sqlx(StatusCode, String),
    // Error::NotFound is what we'll use to conveniently map response to HTTP 404s.
    NotFound
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        match err {
            // For queries that can't find matching rows, we return an HTTP 404
            sqlx::Error::RowNotFound => Error::NotFound,
            _ => Error::Sqlx(
                // For all other SQLx errors, we return n HTTP 500
                StatusCode::INTERNAL_SERVER_ERROR,
                // We include the string returned by the SQLx error in the response body of our 500s.
                err.to_string(),
            ),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            // (StatusCode, String) because axum provides an implementation of IntoResponse for us.
            Error::Sqlx(code, body) => (code, body).into_response(),
            // Call into_response() on StatusCode::NOT_FOUND, which gives us an empty HTTP 404 response
            Error::NotFound => StatusCode::NOT_FOUND.into_response(),
        }
    }
}