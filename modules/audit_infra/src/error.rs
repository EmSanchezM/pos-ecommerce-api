use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuditInfraError {
    #[error("Audit log database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Audit log serialization error: {0}")]
    Serialization(String),

    #[error("Audit log error: {0}")]
    Internal(String),
}
