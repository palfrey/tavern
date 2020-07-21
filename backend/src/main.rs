use actix::{Actor, StreamHandler};
use actix_files::NamedFile;
use actix_http;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::result::Result as StdResult;
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
    Actix {
        #[from]
        source: actix_http::error::Error,
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

struct TavernWs;

impl Actor for TavernWs {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<StdResult<ws::Message, ws::ProtocolError>> for TavernWs {
    fn handle(&mut self, msg: StdResult<ws::Message, ws::ProtocolError>, _ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                println!("msg: {:?}", msg);
            }
            Ok(ws::Message::Text(text)) => {
                println!("text: {:?}", text);
            }
            Ok(ws::Message::Binary(bin)) => {
                println!("bin: {:?}", bin);
            }
            other => println!("Something else: {:?}", other),
        }
    }
}

async fn websocket(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse> {
    let resp = ws::start(TavernWs {}, &req, stream);
    println!("{:?}", resp);
    resp.map_err(|e| e.into())
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
            .route("/ws", web::get().to(websocket))
            .route("/{filename:.*}", web::get().to(index))
    })
    .bind("0.0.0.0:5000")?
    .run()
    .await
}
