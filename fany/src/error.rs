#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Lua error: {0}")]
    LuaError(#[from] mlua::Error),

    #[error("Script parsing error: {0}")]
    ScriptParseError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Schema error: {0}")]
    SchemaError(#[from] SchemaError),
}

#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    #[error("Domain not allowed: {0}")]
    NotAllowedDomain(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Invalid url: {0}")]
    InvalidUrl(String),
}

pub type StdResult<T, E> = std::result::Result<T, E>;

pub type Result<T> = std::result::Result<T, Error>;

pub type SchemaResult<T> = std::result::Result<T, SchemaError>;
