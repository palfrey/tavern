mod commands;
mod error;
mod types;

use crate::error::{MyError, Result};
use crate::types::{Client, Person};
use actix_files::NamedFile;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::{env, io, path::PathBuf};
use uuid::Uuid;

async fn websocket(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse> {
    let id_str = req.match_info().query("id");
    let id = Uuid::parse_str(id_str)?;
    let person = Person {
        id,
        name: String::from(""),
        pub_id: None,
        table_id: None,
    };
    commands::PEOPLE.write().insert(id, person);
    let resp = ws::start(Client { id }, &req, stream);
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

    HttpServer::new(|| {
        App::new()
            .route("/ws/{id}", web::get().to(websocket))
            .route("/{filename:.*}", web::get().to(index))
    })
    .bind("0.0.0.0:5000")?
    .run()
    .await
}
