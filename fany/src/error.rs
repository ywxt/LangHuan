#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Lua error: {0}")]
    LuaError(#[from] mlua::Error),

    #[error("Script parsing error: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
