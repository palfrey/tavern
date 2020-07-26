use crate::schema::{person, pub_table, public_house};
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::{AsChangeset, Insertable, Queryable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

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

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Insertable, AsChangeset)]
#[table_name = "person"]
#[changeset_options(treat_none_as_null = "true")]
pub struct Person {
    pub id: Uuid,
    pub name: Option<String>,
    pub pub_id: Option<Uuid>,
    pub table_id: Option<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Insertable)]
#[table_name = "pub_table"]
pub struct PubTable {
    pub id: Uuid,
    pub name: String,
    //pub people: Vec<Uuid>,
    pub pub_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Insertable)]
#[table_name = "public_house"]
pub struct Pub {
    pub id: Uuid,
    pub name: String,
    //pub people: Vec<Uuid>,
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
    Table { data: PubTable },
    Tables { list: Vec<PubTable> },
    Person { data: Person },
    Data { author: Uuid, content: String },
    Pong,
}
