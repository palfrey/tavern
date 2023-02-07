mod commands;
mod db;
mod error;
mod migrations;
mod types;

use crate::error::{MyError, Result};
use crate::types::{Client, Person};
use actix_files::NamedFile;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use anyhow::Context;
use log::info;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
use std::{env, io, path::PathBuf};
use uuid::Uuid;

async fn websocket(
    req: HttpRequest,
    stream: web::Payload,
    pool: web::Data<types::Pool>,
) -> Result<HttpResponse> {
    let id_str = req.match_info().query("id");
    let id = Uuid::parse_str(id_str)?;
    let mut conn = pool.get_ref().get()?;
    Person::add_person(&mut conn, id)?;
    let resp = ws::start(
        Client {
            id,
            pool: pool.get_ref().clone(),
        },
        &req,
        stream,
    );
    info!("Resp: {resp:?}");
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
    Ok(NamedFile::open(&path).context(format!("Opening {:?}", &path))?)
}

fn main() -> io::Result<()> {
    env_logger::init();
    let pool = db::make_pool();
    migrations::run_migrations(&mut pool.get().unwrap());

    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let thread_pair = Arc::clone(&pair);

    let thread_pool = pool.clone();
    thread::spawn(move || {
        let (lock, cvar) = &*thread_pair;
        let mut conn = thread_pool.get().unwrap();
        loop {
            Person::cleanup_outdated(&mut conn).unwrap();
            info!("Cleanup done");
            let started = lock.lock().unwrap();
            let result = cvar.wait_timeout(started, Duration::from_secs(60)).unwrap();
            if !result.1.timed_out() {
                break;
            }
        }
    });

    actix::run(async {
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/ws/{id}", web::get().to(websocket))
                .route("/{filename:.*}", web::get().to(index))
        })
        .bind("0.0.0.0:5000")
        .unwrap()
        .run()
        .await
        .unwrap()
    })?;
    pair.1.notify_one();
    Ok(())
}
