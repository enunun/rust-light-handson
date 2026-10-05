use thiserror::Error;

/// `rgit`の操作が失敗した理由．
#[derive(Debug, Error)]
pub enum Error {
    #[error("not a git repository (or any of the parent directories): .git")]
    NotARepository,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Usage(#[from] clap::Error),
}
