mod commands;
mod db;
mod error;
mod migrations;
mod types;

use crate::types::{Client, Person};
use log::info;
use std::convert::Infallible;
use std::io;
use std::net::SocketAddr;
use std::ops::DerefMut;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::task;
use tokio::time::timeout;
use uuid::Uuid;
use warp::ws::WebSocket;
use warp::Filter;

async fn websocket(id_str: String, ws: WebSocket, pool: types::Pool) {
    info!("starting websocket");
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

    let notifier = Arc::new(Notify::new());
    let thread_notifier = Arc::clone(&notifier);

    let thread_pool = pool.clone();
    task::spawn(async move {
        loop {
            info!("Start cleanup");
            let mut conn = thread_pool.get().await.unwrap();
            Person::cleanup_outdated(&mut conn).await.unwrap();
            info!("Cleanup done");
            if timeout(Duration::from_secs(60), thread_notifier.notified()).await.is_ok() { 
                break;
            }
        }
    });

    let ws = warp::path!("ws" / String)
        .and(warp::ws())
        .and(with_db(pool.clone()))
        .map(|id: String, ws: warp::ws::Ws, pool: types::Pool| {
            info!("WS");
            ws.on_upgrade(move |socket| websocket(id, socket, pool))
        });
    let routes = ws;

    warp::serve(routes)
        .run("0.0.0.0:5000".parse::<SocketAddr>().unwrap())
        .await;
    notifier.notify_one();
    Ok(())
}
