use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    #[error("Mapping error: {0}")]
    MappingError(String),
}

pub type DbResult<T> = Result<T, DbError>;

// Convert DbError to DomainError
impl From<DbError> for domain::errors::DomainError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::NotFound(msg) => domain::errors::DomainError::NotFound(msg),
            DbError::DuplicateEntry(msg) => domain::errors::DomainError::DuplicateEntry(msg),
            DbError::DatabaseError(e) => {
                domain::errors::DomainError::InternalError(format!("Database error: {}", e))
            }
            DbError::MappingError(msg) => {
                domain::errors::DomainError::InternalError(format!("Mapping error: {}", msg))
            }
        }
    }
}
