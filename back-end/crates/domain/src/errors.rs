use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Player already drafted: {0}")]
    PlayerAlreadyDrafted(String),
}

pub type DomainResult<T> = Result<T, DomainError>;
