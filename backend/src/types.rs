use bb8_postgres::PostgresConnectionManager;
use chrono::NaiveDateTime;
use postgres::NoTls;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type Pool = bb8::Pool<PostgresConnectionManager<NoTls>>;
pub type DbConnection<'a> = bb8::PooledConnection<'a, PostgresConnectionManager<NoTls>>;

#[derive(Clone)]
pub struct Client {
    pub id: Uuid,
    pub pool: Pool,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Client {}>", self.id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pub {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PubWithPeople {
    pub id: Uuid,
    pub name: String,
    pub persons: Vec<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PubTable {
    pub id: Uuid,
    pub name: String,
    pub pub_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TableWithPeople {
    pub id: Uuid,
    pub name: String,
    pub pub_id: Uuid,
    pub persons: Vec<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Person {
    pub id: Uuid,
    pub name: Option<String>,
    pub pub_id: Option<Uuid>,
    pub table_id: Option<Uuid>,
    pub last_updated: NaiveDateTime,
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
    DeletePub { pub_id: Uuid },
    CreateTable { pub_id: Uuid, name: String },
    ListTables { pub_id: Uuid },
    JoinTable { table_id: Uuid },
    DeleteTable { table_id: Uuid },
    LeaveTable,
    Send { user_id: Uuid, content: String },
    Ping,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum Response {
    CreatePub { data: PubWithPeople },
    Pubs { list: Vec<PubWithPeople> },
    CreateTable { data: TableWithPeople },
    Tables { list: Vec<TableWithPeople> },
    Person { data: Person },
    Data { author: Uuid, content: String },
    Pong,
}
