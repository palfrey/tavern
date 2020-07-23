use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Client {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Person {
    pub id: Uuid,
    pub name: String,
    pub pub_id: Option<Uuid>,
    pub table_id: Option<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Table {
    pub id: Uuid,
    pub name: String,
    pub people: Vec<Uuid>,
    pub pub_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pub {
    pub id: Uuid,
    pub name: String,
    pub people: Vec<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "kind")]
pub enum Command {
    ListPubs,
    SetName { name: String },
    GetPerson { user_id: Uuid },
    CreatePub { name: String },
    LeavePub,
    JoinPub { pub_id: Uuid },
    CreateTable { pub_id: Uuid, name: String },
    ListTables { pub_id: Uuid },
    JoinTable { table_id: Uuid },
    LeaveTable,
    Send { user_id: Uuid, content: String },
    Ping,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum Response {
    Pub { data: Pub },
    Pubs { list: Vec<Pub> },
    Table { data: Table },
    Tables { list: Vec<Table> },
    Person { data: Person },
    Data { author: Uuid, content: String },
    Pong,
}
