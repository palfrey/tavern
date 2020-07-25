use crate::error::Result;
use crate::types::{Client, Command, Person, Pub, Response, Table};
use actix::prelude::AsyncContext;
use actix::{Actor, Addr, Handler, Message, StreamHandler};
use actix_web_actors::ws;
use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::result::Result as StdResult;
use uuid::Uuid;

lazy_static! {
    pub static ref PEOPLE: RwLock<HashMap<Uuid, Person>> = RwLock::new(HashMap::new());
    static ref PUBS: RwLock<HashMap<Uuid, Pub>> = RwLock::new(HashMap::new());
    static ref TABLES: RwLock<HashMap<Uuid, Table>> = RwLock::new(HashMap::new());
    static ref ADDRS: RwLock<HashMap<Uuid, Addr<Client>>> = RwLock::new(HashMap::new());
}

impl Actor for Client {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ADDRS.write().insert(self.id, ctx.address());
    }
}

impl Client {
    fn leave_pub(&mut self) {
        let mut people = PEOPLE.write();
        let person = people.get_mut(&self.id).unwrap();
        if let Some(pub_id) = person.pub_id {
            PUBS.write()
                .get_mut(&pub_id)
                .unwrap()
                .people
                .retain(|p| p != &self.id);
        }
        person.pub_id = None;
    }

    fn leave_table(&mut self) {
        let mut people = PEOPLE.write();
        let person = people.get_mut(&self.id).unwrap();
        if let Some(table_id) = person.table_id {
            TABLES
                .write()
                .get_mut(&table_id)
                .unwrap()
                .people
                .retain(|p| p != &self.id);
        }
        person.table_id = None;
    }

    fn return_self(&self, ctx: &mut <Client as Actor>::Context)
    where
        Self: Actor,
    {
        ctx.text(
            serde_json::to_string(&Response::Person {
                data: PEOPLE.read().get(&self.id).unwrap().clone(),
            })
            .unwrap(),
        );
    }
}

struct ClientMsg {
    payload: String,
}

impl Message for ClientMsg {
    type Result = Result<()>;
}

impl Handler<ClientMsg> for Client {
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

impl StreamHandler<StdResult<ws::Message, ws::ProtocolError>> for Client {
    fn handle(&mut self, msg: StdResult<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                println!("msg: {:?}", msg);
            }
            Ok(ws::Message::Text(text)) => {
                match serde_json::from_str::<Command>(&text) {
                    Ok(cmd) => {
                        println!("command: {:?}", cmd);
                        match cmd {
                            Command::ListPubs => {
                                ctx.text(
                                    serde_json::to_string(&Response::Pubs {
                                        list: PUBS.read().values().cloned().collect::<Vec<Pub>>(),
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
                                PUBS.write().insert(pub_id, new_pub.clone());
                                PEOPLE.write().get_mut(&self.id).unwrap().pub_id = Some(pub_id);
                                ctx.text(
                                    serde_json::to_string(&Response::Pub { data: new_pub })
                                        .unwrap(),
                                );
                            }
                            Command::JoinPub { pub_id } => {
                                // Only allowed to be in one pub
                                self.leave_table();
                                self.leave_pub();
                                PUBS.write().get_mut(&pub_id).unwrap().people.push(self.id);
                                PEOPLE.write().get_mut(&self.id).unwrap().pub_id = Some(pub_id);
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
                                TABLES.write().insert(pub_id, new_table.clone());
                                PEOPLE.write().get_mut(&self.id).unwrap().table_id = Some(table_id);
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
                                    .get_mut(&table_id)
                                    .unwrap()
                                    .people
                                    .push(self.id);
                                PEOPLE.write().get_mut(&self.id).unwrap().table_id = Some(table_id);
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
                                            .values()
                                            .filter(|t| t.pub_id == pub_id)
                                            .cloned()
                                            .collect::<Vec<Table>>(),
                                    })
                                    .unwrap(),
                                );
                            }
                            Command::Send { user_id, content } => {
                                ADDRS
                                    .read()
                                    .get(&user_id)
                                    .unwrap()
                                    .try_send(ClientMsg { payload: content })
                                    .unwrap();
                            }
                            Command::SetName { name } => {
                                PEOPLE.write().get_mut(&self.id).unwrap().name = name;
                                self.return_self(ctx);
                            }
                            Command::GetPerson { user_id } => {
                                ctx.text(
                                    serde_json::to_string(&Response::Person {
                                        data: PEOPLE.read().get(&user_id).unwrap().clone(),
                                    })
                                    .unwrap(),
                                );
                            }
                            Command::Ping => {
                                ctx.text(serde_json::to_string(&Response::Pong).unwrap());
                            }
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
