use crate::error::{MyError, Result};
use crate::schema::person::dsl as person_dsl;
use crate::schema::person::dsl::person;
use crate::schema::pub_table::dsl as table_dsl;
use crate::schema::pub_table::dsl::pub_table;
use crate::schema::public_house::dsl::public_house;
use crate::types::{DbConnection, Person, Pool, Pub, PubTable};
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::ExpressionMethods;
use diesel::{QueryDsl, RunQueryDsl};
use diesel_migrations::embed_migrations;
use std::env;
use std::result::Result as StdResult;
use uuid::Uuid;

embed_migrations!();

pub fn make_pool() -> Pool {
    let db_url = env::var("DATABASE_URL").expect("Database url not set");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = Pool::new(manager).expect("Failed to create db pool");
    let conn = pool.get().unwrap();
    embedded_migrations::run(&conn).unwrap();
    pool
}

fn map_empty<T, E>(res: StdResult<T, E>) -> Result<()>
where
    E: Into<MyError>,
{
    res.map(|_| ()).map_err(|e| e.into())
}

impl Person {
    pub fn leave_pub(conn: &DbConnection, person_id: Uuid) -> Result<()> {
        map_empty(
            diesel::update(person.filter(person_dsl::id.eq(person_id)))
                .set(person_dsl::pub_id.eq::<Option<Uuid>>(None))
                .execute(conn),
        )
    }

    pub fn leave_table(conn: &DbConnection, person_id: Uuid) -> Result<()> {
        map_empty(
            diesel::update(person.filter(person_dsl::id.eq(person_id)))
                .set(person_dsl::table_id.eq::<Option<Uuid>>(None))
                .execute(conn),
        )
    }

    pub fn load_from_db(conn: &DbConnection, person_id: Uuid) -> Result<Person> {
        person
            .filter(person_dsl::id.eq(person_id))
            .first(conn)
            .map_err(|e| e.into())
    }

    pub fn add_person(conn: &DbConnection, person_id: Uuid) -> Result<()> {
        map_empty(
            diesel::insert_into(person)
                .values(&Person {
                    id: person_id,
                    name: None,
                    pub_id: None,
                    table_id: None,
                })
                .on_conflict_do_nothing()
                .execute(conn),
        )
    }

    pub fn set_name(conn: &DbConnection, person_id: Uuid, name: String) -> Result<()> {
        map_empty(
            diesel::update(person.filter(person_dsl::id.eq(person_id)))
                .set(person_dsl::name.eq(name))
                .execute(conn),
        )
    }

    pub fn set_pub(conn: &DbConnection, person_id: Uuid, pub_id: Uuid) -> Result<()> {
        map_empty(
            diesel::update(person.filter(person_dsl::id.eq(person_id)))
                .set(person_dsl::pub_id.eq(pub_id))
                .execute(conn),
        )
    }

    pub fn set_table(conn: &DbConnection, person_id: Uuid, table_id: Uuid) -> Result<()> {
        map_empty(
            diesel::update(person.filter(person_dsl::id.eq(person_id)))
                .set(person_dsl::table_id.eq(table_id))
                .execute(conn),
        )
    }
}

impl Pub {
    pub fn get_pubs(conn: &DbConnection) -> Result<Vec<Pub>> {
        public_house.load::<Pub>(conn).map_err(|e| e.into())
    }

    pub fn add_pub(&self, conn: &DbConnection) -> Result<()> {
        map_empty(diesel::insert_into(public_house).values(self).execute(conn))
    }
}

impl PubTable {
    pub fn get_tables(conn: &DbConnection, pub_id: Uuid) -> Result<Vec<PubTable>> {
        pub_table
            .filter(table_dsl::pub_id.eq(pub_id))
            .load::<PubTable>(conn)
            .map_err(|e| e.into())
    }

    pub fn add_table(&self, conn: &DbConnection) -> Result<()> {
        map_empty(diesel::insert_into(pub_table).values(self).execute(conn))
    }
}
