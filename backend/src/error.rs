use std::{env, fmt, io};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, MyError>;

#[derive(Debug, Error)]
pub enum MyError {
    Io {
        #[from]
        source: io::Error,
    },
    Env {
        #[from]
        source: env::VarError,
    },
    Uuid {
        #[from]
        source: uuid::ParseError,
    },
    Actix {
        content: String,
    },
    R2D2 {
        #[from]
        source: r2d2::Error,
    },
    Postgres {
        #[from]
        source: postgres::Error,
    },
    #[error(transparent)]
    Other(#[from] anyhow::Error), // source and Display delegate to anyhow::Error
}
impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl actix_web::error::ResponseError for MyError {}
