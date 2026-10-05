use thiserror::Error;

/// `rgit`の操作が失敗した理由．
#[derive(Debug, Error)]
pub enum Error {
    #[error("not a git repository (or any of the parent directories): .git")]
    NotARepository,
    #[error("Not a valid object name {0}")]
    ObjectNotFound(String),
    #[error("short object ID {0} is ambiguous")]
    AmbiguousObject(String),
    #[error("corrupt object: {0}")]
    CorruptObject(&'static str),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Usage(#[from] clap::Error),
}
