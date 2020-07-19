use actix_files::NamedFile;
use actix_web::HttpRequest;
use std::{env, fmt, io, path::PathBuf};
use thiserror::Error;

type Result<T> = std::result::Result<T, MyError>;

#[derive(Debug, Error)]
enum MyError {
    Io {
        #[from]
        source: io::Error,
    },
    Env {
        #[from]
        source: env::VarError,
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

async fn index(req: HttpRequest) -> Result<NamedFile> {
    let mut path = PathBuf::from(env::var("FRONTEND")?);
    let filename = match req.match_info().query("filename") {
        "" => "index.html",
        other => other,
    };
    path.push(filename);
    Ok(NamedFile::open(path)?)
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    use actix_web::{web, App, HttpServer};

    HttpServer::new(|| App::new().route("/{filename:.*}", web::get().to(index)))
        .bind("0.0.0.0:5000")?
        .run()
        .await
}
