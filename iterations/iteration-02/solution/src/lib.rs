pub mod cli;
mod error;
mod object;
mod oid;
mod repo;

pub use error::Error;
pub use object::hash_blob;
pub use oid::{ObjectId, ParseObjectIdError};
