use crate::error::Result;
use crate::types::{
    Client, Command, DbConnection, Person, Pub, PubTable, PubWithPeople, Response, TableWithPeople,
};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use lazy_static::lazy_static;
use log::{debug, info, warn};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::{Message, WebSocket};

lazy_static! {
    static ref ADDRS: DashMap<Uuid, UnboundedSender<Message>> = DashMap::new();
}

impl Client {
    pub async fn run_user(&self, ws: WebSocket) {
        let (mut user_ws_tx, mut user_ws_rx) = ws.split();

        // Use an unbounded channel to handle buffering and flushing of messages
        // to the websocket...
        let (tx, rx) = mpsc::unbounded_channel();
        let mut rx = UnboundedReceiverStream::new(rx);

        tokio::task::spawn(async move {
            while let Some(message) = rx.next().await {
                user_ws_tx
                    .send(message)
                    .unwrap_or_else(|e| {
                        warn!("websocket send error: {}", e);
                    })
                    .await;
            }
        });

        ADDRS.insert(self.id, tx);

        while let Some(result) = user_ws_rx.next().await {
            let msg = match result {
                Ok(msg) => msg,
                Err(e) => {
                    warn!("websocket error(uid={}): {}", self.id, e);
                    break;
                }
            };
            self.handle_msg(msg).await;
        }

        info!("Disconnected: {}", self.id);

        // user_ws_rx stream will keep processing as long as the user stays
        // connected. Once they disconnect, then...
        // user_disconnected(my_id, &users).await;
    }

    async fn send_text<S: Into<String> + std::fmt::Display>(&self, msg: S) {
        ADDRS.get(&self.id).and_then(|tx| {
            debug!("send_text: {}", msg);
            tx.send(Message::text(msg)).unwrap();
            Some(())
        });
    }

    async fn leave_pub<'a>(&self, conn: &mut DbConnection<'a>) -> Result<()> {
        Person::leave_pub(conn, self.id).await
    }

    async fn leave_table<'a>(&self, conn: &mut DbConnection<'a>) -> Result<()> {
        Person::leave_table(conn, self.id).await
    }

    async fn return_self<'a>(&self, conn: &mut DbConnection<'a>) {
        self.send_text(
            serde_json::to_string(&Response::Person {
                data: Person::load_from_db(conn, self.id).await.unwrap(),
            })
            .unwrap(),
        )
        .await;
    }

    async fn send_tables<'a>(&self, conn: &mut DbConnection<'a>, pub_id: Uuid) {
        self.send_text(
            serde_json::to_string(&Response::Tables {
                list: PubTable::get_tables(conn, pub_id).await.unwrap(),
            })
            .unwrap(),
        )
        .await;
    }

    async fn handle_msg(&self, msg: Message) {
        if msg.is_ping() {
            println!("msg: {msg:?}");
        } else if msg.is_text() {
            let text = msg.to_str().unwrap();
            match serde_json::from_str::<Command>(&text) {
                Ok(cmd) => {
                    println!("command: {cmd:?}");
                    let mut conn = self.pool.get().await.unwrap();
                    match cmd {
                        Command::ListPubs => {
                            let pubs = serde_json::to_string(&Response::Pubs {
                                list: Pub::get_pubs(&mut conn).await.unwrap(),
                            })
                            .unwrap();
                            self.send_text(pubs).await;
                        }
                        Command::CreatePub { name } => {
                            self.leave_table(&mut conn).await.unwrap();
                            self.leave_pub(&mut conn).await.unwrap();
                            let pub_id = Uuid::new_v4();
                            let new_pub = Pub {
                                id: pub_id,
                                name: name.clone(),
                            };
                            new_pub.add_pub(&mut conn).await.unwrap();
                            Person::set_pub(&mut conn, self.id, pub_id).await.unwrap();
                            self.send_text(
                                serde_json::to_string(&Response::CreatePub {
                                    data: PubWithPeople {
                                        id: pub_id,
                                        name,
                                        persons: vec![self.id],
                                    },
                                })
                                .unwrap(),
                            )
                            .await;
                            self.return_self(&mut conn).await;
                        }
                        Command::DeletePub { pub_id } => {
                            Pub::delete_pub(&mut conn, pub_id).await.unwrap();
                            let pubs = serde_json::to_string(&Response::Pubs {
                                list: Pub::get_pubs(&mut conn).await.unwrap(),
                            })
                            .unwrap();
                            self.send_text(pubs).await;
                        }
                        Command::JoinPub { pub_id } => {
                            // Only allowed to be in one pub
                            self.leave_table(&mut conn).await.unwrap();
                            self.leave_pub(&mut conn).await.unwrap();
                            Person::set_pub(&mut conn, self.id, pub_id).await.unwrap();
                            self.return_self(&mut conn).await;
                            self.send_tables(&mut conn, pub_id).await;
                        }
                        Command::CreateTable { pub_id, name } => {
                            self.leave_table(&mut conn).await.unwrap();
                            let table_id = Uuid::new_v4();
                            let new_table = PubTable {
                                id: table_id,
                                pub_id,
                                name: name.clone(),
                            };
                            new_table.add_table(&mut conn).await.unwrap();
                            Person::set_table(&mut conn, self.id, table_id)
                                .await
                                .unwrap();
                            self.send_text(
                                serde_json::to_string(&Response::CreateTable {
                                    data: TableWithPeople {
                                        id: table_id,
                                        pub_id,
                                        name,
                                        persons: vec![self.id],
                                    },
                                })
                                .unwrap(),
                            )
                            .await;
                            self.return_self(&mut conn).await;
                        }
                        Command::JoinTable { table_id } => {
                            // Only allowed to be in one pub
                            self.leave_table(&mut conn).await.unwrap();
                            Person::set_table(&mut conn, self.id, table_id)
                                .await
                                .unwrap();

                            self.return_self(&mut conn).await;
                        }
                        Command::LeavePub | Command::LeaveTable => {
                            self.leave_table(&mut conn).await.unwrap();
                            if cmd == Command::LeavePub {
                                self.leave_pub(&mut conn).await.unwrap();
                            }
                            self.return_self(&mut conn).await;
                        }
                        Command::ListTables { pub_id } => {
                            self.send_tables(&mut conn, pub_id).await;
                        }
                        Command::Send { user_id, content } => {
                            match ADDRS.get(&user_id) {
                                Some(addr) => {
                                    addr.send(Message::text(
                                        serde_json::to_string(&Response::Data {
                                            author: self.id,
                                            content,
                                        })
                                        .unwrap(),
                                    ))
                                    .unwrap();
                                }
                                None => {
                                    println!("Can't send to {}. Available addrs", user_id);
                                }
                            };
                        }
                        Command::SetName { name } => {
                            Person::set_name(&mut conn, self.id, name).await.unwrap();
                            self.return_self(&mut conn).await;
                        }
                        Command::GetPerson { user_id } => {
                            let person = serde_json::to_string(&Response::Person {
                                data: Person::load_from_db(&mut conn, user_id).await.unwrap(),
                            })
                            .unwrap();
                            self.send_text(person).await;
                        }
                        Command::DeleteTable { table_id } => {
                            let pub_id = PubTable::delete_table(&mut conn, table_id).await.unwrap();
                            self.send_tables(&mut conn, pub_id).await;
                        }
                        Command::Ping => {
                            Person::update_last(&mut conn, self.id).await.unwrap();
                            self.send_text(serde_json::to_string(&Response::Pong).unwrap())
                                .await;
                        }
                    }
                }
                Err(_error) => {
                    println!("Error parsing command: {text}");
                }
            }
        } else if msg.is_binary() {
            println!("bin: {msg:?}");
        } else {
            println!("Something else: {msg:?}")
        }
    }
}
