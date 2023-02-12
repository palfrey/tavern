mod commands;
mod db;
mod error;
mod migrations;
mod types;

use crate::types::{Client, Person};
use anyhow::Context;
use log::info;
use std::convert::Infallible;
use std::fs;
use std::net::SocketAddr;
use std::ops::DerefMut;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use std::{env, io, path::PathBuf};
use tokio::task;
use uuid::Uuid;
use warp::ws::WebSocket;
use warp::Filter;

async fn websocket(id_str: String, ws: WebSocket, pool: types::Pool) {
    let id = Uuid::parse_str(&id_str).unwrap();
    let conn = pool.get().await.unwrap();
    Person::add_person(&conn, id).await.unwrap();
    info!("Connected for {}", id_str);
    Client {
        id,
        pool: pool.clone(),
    }
    .run_user(ws)
    .await;
}

async fn index(
    query_filename: String,
) -> ::std::result::Result<Box<dyn warp::Reply>, warp::Rejection> {
    let mut path = PathBuf::from(env::var("FRONTEND").unwrap());
    let filename = match query_filename.as_str() {
        "" => "index.html",
        other => other,
    };
    path.push(filename);
    Ok(Box::new(
        fs::read_to_string(&path)
            .context(format!("Opening {:?}", &path))
            .unwrap(),
    ))
}

fn with_db(pool: types::Pool) -> impl Filter<Extract = (types::Pool,), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init();

    let pool = db::make_pool().await;

    let runner = migrations::migrations::runner();
    runner
        .run_async(pool.get().await.unwrap().deref_mut())
        .await
        .unwrap();

    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let thread_pair = Arc::clone(&pair);

    let thread_pool = pool.clone();
    task::spawn(async move {
        let (lock, cvar) = &*thread_pair;
        let mut conn = thread_pool.get().await.unwrap();
        loop {
            Person::cleanup_outdated(&mut conn).await.unwrap();
            info!("Cleanup done");
            let started = lock.lock().unwrap();
            let result = cvar.wait_timeout(started, Duration::from_secs(60)).unwrap();
            if !result.1.timed_out() {
                break;
            }
        }
    });

    let files = warp::path!(String).and_then(index);
    let ws = warp::path!("ws" / String)
        .and(warp::ws())
        .and(with_db(pool.clone()))
        .map(|id: String, ws: warp::ws::Ws, pool: types::Pool| {
            ws.on_upgrade(move |socket| websocket(id, socket, pool))
        });
    let routes = files.or(ws);

    warp::serve(routes)
        .run("0.0.0.0:5000".parse::<SocketAddr>().unwrap())
        .await;
    pair.1.notify_one();
    Ok(())
}
