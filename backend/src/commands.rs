use crate::types::{
    Command, Connection, DbConnection, Person, Pub, PubTable, PubWithPeople, Response,
    TableWithPeople,
};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use lazy_static::lazy_static;
use log::{debug, info, warn};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::{Message, WebSocket};

type ClientChannel = UnboundedSender<Message>;
struct Client {
    sender: ClientChannel,
    pub_id: Option<Uuid>,
    table_id: Option<Uuid>,
}

lazy_static! {
    static ref CLIENTS: DashMap<Uuid, Client> = DashMap::new();
}

fn get_clients_in_pub(pub_id: Uuid) -> Vec<ClientChannel> {
    let mut ret = vec![];
    for client in CLIENTS.iter() {
        if client.pub_id.map(|p| p == pub_id).unwrap_or(false) {
            ret.push(client.sender.clone());
        }
    }
    return ret;
}

fn get_clients_in_table(table_id: Uuid) -> Vec<ClientChannel> {
    let mut ret = vec![];
    for client in CLIENTS.iter() {
        if client.table_id.map(|t| t == table_id).unwrap_or(false) {
            ret.push(client.sender.clone());
        }
    }
    return ret;
}

fn send_text_to_list<S: Into<String> + std::fmt::Display>(
    destinations: &Vec<ClientChannel>,
    msg: S,
) {
    let wrapped = Message::text(msg);
    for dest in destinations {
        dest.send(wrapped.clone()).unwrap();
    }
}

impl Connection {
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

        CLIENTS.insert(
            self.id,
            Client {
                sender: tx,
                pub_id: None,
                table_id: None,
            },
        );

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
        if let Some(client) = CLIENTS.get(&self.id) {
            debug!("send_text: {}", msg);
            client.sender.send(Message::text(msg)).unwrap();
        };
    }

    async fn get_self<'a>(&self, conn: &mut DbConnection<'a>) -> String {
        serde_json::to_string(&Response::Person {
            data: Person::load_from_db(conn, self.id).await.unwrap(),
        })
        .unwrap()
    }

    async fn set_pub<'a>(&self, conn: &mut DbConnection<'a>, pub_id: Option<Uuid>) -> Option<Uuid> {
        if let Some(mut client) = CLIENTS.get_mut(&self.id) {
            if client.table_id.is_some() {
                Person::leave_table(conn, self.id).await.unwrap();
                client.table_id = None;
            }
            if let Some(unwrapped_pub_id) = pub_id {
                Person::set_pub(conn, self.id, unwrapped_pub_id)
                    .await
                    .unwrap();
            } else {
                Person::leave_pub(conn, self.id).await.unwrap();
            }
            let ret = client.pub_id;
            client.pub_id = pub_id;
            ret
        } else {
            None
        }
    }

    async fn set_table<'a>(
        &self,
        conn: &mut DbConnection<'a>,
        table_id: Option<Uuid>,
    ) -> Option<Uuid> {
        if let Some(mut client) = CLIENTS.get_mut(&self.id) {
            if let Some(unwrapped_table_id) = table_id {
                Person::set_table(conn, self.id, unwrapped_table_id)
                    .await
                    .unwrap();
            } else {
                Person::leave_table(conn, self.id).await.unwrap();
            }

            let ret = client.table_id;
            client.table_id = table_id;
            ret
        } else {
            None
        }
    }

    async fn return_self<'a>(&self, conn: &mut DbConnection<'a>) {
        self.send_text(self.get_self(conn).await).await;
    }

    async fn tell_pub_about_self<'a>(&self, conn: &mut DbConnection<'a>) {
        if let Some(client) = CLIENTS.get(&self.id) {
            if let Some(pub_id) = client.pub_id {
                let destinations = get_clients_in_pub(pub_id);
                if destinations.len() == 0 {
                    self.return_self(conn).await;
                } else {
                    send_text_to_list(&destinations, self.get_self(conn).await);
                    let tables = serde_json::to_string(&Response::Tables {
                        list: PubTable::get_tables(conn, pub_id).await.unwrap(),
                    })
                    .unwrap();
                    send_text_to_list(&destinations, tables);
                }
            }
        }
    }

    async fn return_self_to_list<'a>(
        &self,
        conn: &mut DbConnection<'a>,
        destinations: Vec<ClientChannel>,
    ) {
        if destinations.len() == 0 {
            self.return_self(conn).await;
        } else {
            send_text_to_list(&destinations, self.get_self(conn).await);
        }
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
            match serde_json::from_str::<Command>(text) {
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
                            let pub_id = Uuid::new_v4();
                            let new_pub = Pub {
                                id: pub_id,
                                name: name.clone(),
                            };
                            new_pub.add_pub(&mut conn).await.unwrap();
                            self.set_pub(&mut conn, Some(pub_id)).await;
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
                            let old_pub = self.set_pub(&mut conn, Some(pub_id)).await;
                            self.return_self_to_list(&mut conn, get_clients_in_pub(pub_id))
                                .await;
                            if let Some(old_pub_id) = old_pub {
                                self.return_self_to_list(&mut conn, get_clients_in_pub(old_pub_id))
                                    .await;
                            }
                            self.send_tables(&mut conn, pub_id).await;
                        }
                        Command::CreateTable { pub_id, name } => {
                            let table_id = Uuid::new_v4();
                            let new_table = PubTable {
                                id: table_id,
                                pub_id,
                                name: name.clone(),
                            };
                            new_table.add_table(&mut conn).await.unwrap();
                            self.set_table(&mut conn, Some(table_id)).await;
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
                            let old_table = self.set_table(&mut conn, Some(table_id)).await;
                            self.return_self_to_list(&mut conn, get_clients_in_table(table_id))
                                .await;
                            self.tell_pub_about_self(&mut conn).await;
                            if let Some(old_table_id) = old_table {
                                self.return_self_to_list(
                                    &mut conn,
                                    get_clients_in_table(old_table_id),
                                )
                                .await;
                            }
                        }
                        Command::LeavePub | Command::LeaveTable => {
                            if cmd == Command::LeavePub {
                                let old_pub = self.set_pub(&mut conn, None).await;
                                if let Some(old_pub_id) = old_pub {
                                    self.return_self_to_list(
                                        &mut conn,
                                        get_clients_in_pub(old_pub_id),
                                    )
                                    .await;
                                }
                            } else {
                                let old_table = self.set_table(&mut conn, None).await;
                                if let Some(old_table_id) = old_table {
                                    self.return_self_to_list(
                                        &mut conn,
                                        get_clients_in_table(old_table_id),
                                    )
                                    .await;
                                }
                            }
                        }
                        Command::ListTables { pub_id } => {
                            self.send_tables(&mut conn, pub_id).await;
                        }
                        Command::Send { user_id, content } => {
                            match CLIENTS.get(&user_id) {
                                Some(client) => {
                                    client
                                        .sender
                                        .send(Message::text(
                                            serde_json::to_string(&Response::Data {
                                                author: self.id,
                                                content,
                                            })
                                            .unwrap(),
                                        ))
                                        .unwrap();
                                }
                                None => {
                                    println!("Can't send to {user_id}. Available addrs");
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
