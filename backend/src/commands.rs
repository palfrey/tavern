use crate::error::Result;
use crate::types::DbConnection;
use crate::types::{Client, Command, Person, Pub, PubTable, Response};
use actix::prelude::AsyncContext;
use actix::{Actor, Addr, Handler, Message, StreamHandler};
use actix_web_actors::ws;
use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::result::Result as StdResult;
use uuid::Uuid;

lazy_static! {
    static ref ADDRS: RwLock<HashMap<Uuid, Addr<Client>>> = RwLock::new(HashMap::new());
}

impl Actor for Client {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ADDRS.write().insert(self.id, ctx.address());
    }
}

impl Client {
    fn leave_pub(&mut self, conn: &mut DbConnection) -> Result<()> {
        Person::leave_pub(conn, self.id)
    }

    fn leave_table(&mut self, conn: &mut DbConnection) -> Result<()> {
        Person::leave_table(conn, self.id)
    }

    fn return_self(&self, ctx: &mut <Client as Actor>::Context, conn: &mut DbConnection)
    where
        Self: Actor,
    {
        ctx.text(
            serde_json::to_string(&Response::Person {
                data: Person::load_from_db(conn, self.id).unwrap(),
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
        let mut conn = self.pool.get().unwrap();
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
                                        list: Pub::get_pubs(&mut conn).unwrap(),
                                    })
                                    .unwrap(),
                                );
                            }
                            Command::CreatePub { name } => {
                                self.leave_table(&mut conn).unwrap();
                                self.leave_pub(&mut conn).unwrap();
                                let pub_id = Uuid::new_v4();
                                let new_pub = Pub {
                                    id: pub_id,
                                    name,
                                    //people: vec![self.id],
                                };
                                new_pub.add_pub(&mut conn).unwrap();
                                Person::set_pub(&mut conn, self.id, pub_id).unwrap();
                                ctx.text(
                                    serde_json::to_string(&Response::Pub { data: new_pub })
                                        .unwrap(),
                                );
                            }
                            Command::JoinPub { pub_id } => {
                                // Only allowed to be in one pub
                                self.leave_table(&mut conn).unwrap();
                                self.leave_pub(&mut conn).unwrap();
                                Person::set_pub(&mut conn, self.id, pub_id).unwrap();
                                self.return_self(ctx, &mut conn);
                            }
                            Command::CreateTable { pub_id, name } => {
                                self.leave_table(&mut conn).unwrap();
                                let table_id = Uuid::new_v4();
                                let new_table = PubTable {
                                    id: table_id,
                                    pub_id,
                                    name,
                                };
                                new_table.add_table(&mut conn).unwrap();
                                Person::set_table(&mut conn, self.id, table_id).unwrap();
                                ctx.text(
                                    serde_json::to_string(&Response::Table { data: new_table })
                                        .unwrap(),
                                );
                            }
                            Command::JoinTable { table_id } => {
                                // Only allowed to be in one pub
                                self.leave_table(&mut conn).unwrap();
                                Person::set_table(&mut conn, self.id, table_id).unwrap();
                                self.return_self(ctx, &mut conn);
                            }
                            Command::LeavePub | Command::LeaveTable => {
                                self.leave_table(&mut conn).unwrap();
                                if cmd == Command::LeavePub {
                                    self.leave_pub(&mut conn).unwrap();
                                }
                                self.return_self(ctx, &mut conn);
                            }
                            Command::ListTables { pub_id } => {
                                ctx.text(
                                    serde_json::to_string(&Response::Tables {
                                        list: PubTable::get_tables(&mut conn, pub_id).unwrap(),
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
                                Person::set_name(&mut conn, self.id, name).unwrap();
                                self.return_self(ctx, &mut conn);
                            }
                            Command::GetPerson { user_id } => {
                                ctx.text(
                                    serde_json::to_string(&Response::Person {
                                        data: Person::load_from_db(&mut conn, user_id).unwrap(),
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
