use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    InternalError(String),
    DomainError(domain::errors::DomainError),
}

pub type ApiResult<T> = Result<T, ApiError>;

impl From<domain::errors::DomainError> for ApiError {
    fn from(err: domain::errors::DomainError) -> Self {
        ApiError::DomainError(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            ApiError::DomainError(err) => {
                use domain::errors::DomainError;
                match err {
                    DomainError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
                    DomainError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
                    DomainError::DuplicateEntry(msg) => (StatusCode::CONFLICT, msg),
                    DomainError::InvalidState(msg) => (StatusCode::BAD_REQUEST, msg),
                    DomainError::InternalError(msg) => {
                        tracing::error!("Internal error: {}", msg);
                        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
                    }
                }
            }
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::errors::DomainError;

    #[test]
    fn test_not_found_error() {
        let error = ApiError::NotFound("Team not found".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_bad_request_error() {
        let error = ApiError::BadRequest("Invalid input".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_domain_error_conversion() {
        let domain_err = DomainError::ValidationError("Invalid data".to_string());
        let api_error = ApiError::from(domain_err);
        let response = api_error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_duplicate_entry_error() {
        let domain_err = DomainError::DuplicateEntry("Team exists".to_string());
        let api_error = ApiError::from(domain_err);
        let response = api_error.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }
}
