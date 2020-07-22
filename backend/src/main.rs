mod error;
mod types;

use crate::error::{MyError, Result};
use crate::types::{Command, Person, Pub, Response, Table};
use actix::prelude::AsyncContext;
use actix::{Actor, Handler, Message, StreamHandler};
use actix_files::NamedFile;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use lazy_static::lazy_static;
use std::result::Result as StdResult;
use std::sync::RwLock;
use std::{collections::HashMap, env, io, path::PathBuf};
use uuid::Uuid;

lazy_static! {
    static ref PEOPLE: RwLock<HashMap<Uuid, Person>> = RwLock::new(HashMap::new());
    static ref PUBS: RwLock<HashMap<Uuid, Pub>> = RwLock::new(HashMap::new());
    static ref TABLES: RwLock<HashMap<Uuid, Table>> = RwLock::new(HashMap::new());
}

impl Actor for Person {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.addr = Some(ctx.address());
    }
}

impl Person {
    fn leave_pub(&mut self) {
        if let Some(pub_id) = self.pub_id {
            PUBS.write()
                .unwrap()
                .get_mut(&pub_id)
                .unwrap()
                .people
                .retain(|p| p != &self.id);
            self.pub_id = None;
        }
    }

    fn leave_table(&mut self) {
        if let Some(table_id) = self.table_id {
            TABLES
                .write()
                .unwrap()
                .get_mut(&table_id)
                .unwrap()
                .people
                .retain(|p| p != &self.id);
            self.table_id = None;
        }
    }

    fn return_self(&self, ctx: &mut <types::Person as Actor>::Context)
    where
        Self: Actor,
    {
        ctx.text(serde_json::to_string(&Response::Person { data: self.clone() }).unwrap());
    }
}

struct ClientMsg {
    payload: String,
}

impl Message for ClientMsg {
    type Result = Result<()>;
}

impl Handler<ClientMsg> for Person {
    type Result = Result<()>;

    fn handle(&mut self, msg: ClientMsg, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(
            serde_json::to_string(&Response::Data {
                author: self.id,
                content: msg.payload,
            })
            .unwrap(),
        );
        Ok(())
    }
}

impl StreamHandler<StdResult<ws::Message, ws::ProtocolError>> for Person {
    fn handle(&mut self, msg: StdResult<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                println!("msg: {:?}", msg);
            }
            Ok(ws::Message::Text(text)) => {
                println!("text: {:?}", text);
                match serde_json::from_str::<Command>(&text) {
                    Ok(cmd) => {
                        println!("command: {:?}", cmd);
                        match cmd {
                            Command::ListPubs => {
                                ctx.text(
                                    serde_json::to_string(&Response::Pubs {
                                        list: PUBS
                                            .read()
                                            .unwrap()
                                            .values()
                                            .cloned()
                                            .collect::<Vec<Pub>>(),
                                    })
                                    .unwrap(),
                                );
                            }
                            Command::CreatePub { name } => {
                                self.leave_table();
                                self.leave_pub();
                                let pub_id = Uuid::new_v4();
                                let new_pub = Pub {
                                    id: pub_id,
                                    name,
                                    people: vec![self.id],
                                };
                                PUBS.write().unwrap().insert(pub_id, new_pub.clone());
                                self.pub_id = Some(pub_id);
                                ctx.text(
                                    serde_json::to_string(&Response::Pub { data: new_pub })
                                        .unwrap(),
                                );
                            }
                            Command::JoinPub { pub_id } => {
                                // Only allowed to be in one pub
                                self.leave_table();
                                self.leave_pub();
                                PUBS.write()
                                    .unwrap()
                                    .get_mut(&pub_id)
                                    .unwrap()
                                    .people
                                    .push(self.id);
                                self.pub_id = Some(pub_id);
                                self.return_self(ctx);
                            }
                            Command::CreateTable { pub_id, name } => {
                                self.leave_table();
                                let table_id = Uuid::new_v4();
                                let new_table = Table {
                                    id: table_id,
                                    pub_id,
                                    name,
                                    people: vec![self.id],
                                };
                                TABLES.write().unwrap().insert(pub_id, new_table.clone());
                                self.table_id = Some(table_id);
                                ctx.text(
                                    serde_json::to_string(&Response::Table { data: new_table })
                                        .unwrap(),
                                );
                            }
                            Command::JoinTable { table_id } => {
                                // Only allowed to be in one pub
                                self.leave_table();
                                TABLES
                                    .write()
                                    .unwrap()
                                    .get_mut(&table_id)
                                    .unwrap()
                                    .people
                                    .push(self.id);
                                self.table_id = Some(table_id);
                                self.return_self(ctx);
                            }
                            Command::LeavePub | Command::LeaveTable => {
                                self.leave_table();
                                if cmd == Command::LeavePub {
                                    self.leave_pub();
                                }
                                self.return_self(ctx);
                            }
                            Command::ListTables { pub_id } => {
                                ctx.text(
                                    serde_json::to_string(&Response::Tables {
                                        list: TABLES
                                            .read()
                                            .unwrap()
                                            .values()
                                            .filter(|t| t.pub_id == pub_id)
                                            .cloned()
                                            .collect::<Vec<Table>>(),
                                    })
                                    .unwrap(),
                                );
                            }
                            Command::Send { user_id, content } => {
                                PEOPLE
                                    .read()
                                    .unwrap()
                                    .get(&user_id)
                                    .unwrap()
                                    .addr
                                    .as_ref()
                                    .unwrap()
                                    .try_send(ClientMsg { payload: content })
                                    .unwrap();
                            }
                            other => println!("Unimplemented: {:?}", other),
                        }
                    }
                    Err(_error) => {
                        println!("Error parsing command: {}", text);
                        return;
                    }
                };
            }
            Ok(ws::Message::Binary(bin)) => {
                println!("bin: {:?}", bin);
            }
            other => println!("Something else: {:?}", other),
        }
    }
}

async fn websocket(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse> {
    let id_str = req.match_info().query("id");
    let id = Uuid::parse_str(id_str)?;
    let actor = Person {
        id,
        name: String::from(""),
        pub_id: None,
        table_id: None,
        addr: None,
    };
    let resp = ws::start(actor, &req, stream);
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
