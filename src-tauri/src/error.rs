use std::fmt;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message: String,
}

impl AppError {
    pub(crate) fn database_open() -> Self {
        Self::new("database_open_failed", "Unable to open the local database.")
    }

    pub(crate) fn database_operation() -> Self {
        Self::new(
            "database_operation_failed",
            "The database operation failed.",
        )
    }

    pub(crate) fn backup_failed() -> Self {
        Self::new(
            "backup_failed",
            "Unable to protect the library before updating.",
        )
    }

    pub(crate) fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AppError {}
