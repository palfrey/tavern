mod commands;
mod db;
mod error;
mod schema;
mod types;

use crate::error::{MyError, Result};
use crate::types::{Client, Person};
use actix_files::NamedFile;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::{env, io, path::PathBuf};
use uuid::Uuid;

async fn websocket(
    req: HttpRequest,
    stream: web::Payload,
    pool: web::Data<types::Pool>,
) -> Result<HttpResponse> {
    let id_str = req.match_info().query("id");
    let id = Uuid::parse_str(id_str)?;
    let conn = pool.get_ref().get()?;
    Person::add_person(&conn, id)?;
    let resp = ws::start(
        Client {
            id,
            pool: pool.get_ref().clone(),
        },
        &req,
        stream,
    );
    println!("Resp: {:?}", resp);
    resp.map_err(|e| MyError::Actix {
        content: e.to_string(),
    })
}

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

    let pool = db::make_pool();

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .route("/ws/{id}", web::get().to(websocket))
            .route("/{filename:.*}", web::get().to(index))
    })
    .bind("0.0.0.0:5000")?
    .run()
    .await
}
