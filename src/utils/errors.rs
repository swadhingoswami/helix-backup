use thiserror::Error;

#[derive(Error, Debug)]
pub enum HelixError {
    #[error("Device error: {0}")]
    Device(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Tracking error: {0}")]
    Tracking(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Unsupported: {0}")]
    Unsupported(String),

    #[error("Cancelled")]
    Cancelled,

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl HelixError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, HelixError::Io(_) | HelixError::Device(_))
    }

    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            HelixError::Encryption(_)
                | HelixError::Database(_)
                | HelixError::Repository(_)
        )
    }

    pub fn user_message(&self) -> String {
        match self {
            HelixError::Device(msg) => format!("Device operation failed: {}", msg),
            HelixError::Io(e) => format!("I/O error: {}", e),
            HelixError::Storage(msg) => format!("Storage error: {}", msg),
            HelixError::Tracking(msg) => format!("Change tracking error: {}", msg),
            HelixError::Encryption(msg) => format!("Encryption error: {}", msg),
            HelixError::Configuration(msg) => format!("Configuration error: {}", msg),
            HelixError::Repository(msg) => format!("Repository error: {}", msg),
            HelixError::Serialization(e) => format!("Data format error: {}", e),
            HelixError::Database(e) => format!("Database error: {}", e),
            HelixError::InvalidInput(msg) => format!("Invalid input: {}", msg),
            HelixError::NotFound(msg) => format!("Not found: {}", msg),
            HelixError::PermissionDenied(msg) => format!("Permission denied: {}", msg),
            HelixError::Unsupported(msg) => format!("Unsupported: {}", msg),
            HelixError::Cancelled => "Operation cancelled".to_string(),
            HelixError::Unknown(msg) => format!("Unknown error: {}", msg),
        }
    }
}


