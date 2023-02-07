use crate::error::{MyError, Result};
use crate::types::{DbConnection, Person, Pool, Pub, PubTable, PubWithPeople, TableWithPeople};
use log::warn;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use std::env;
use std::result::Result as StdResult;
use uuid::Uuid;

pub fn make_pool() -> Pool {
    let db_url = env::var("DATABASE_URL").expect("Database url not set");
    let manager = PostgresConnectionManager::new(db_url, TlsMode::None).unwrap();

    Pool::new(manager).expect("Failed to create db pool")
}

fn map_empty<T, E>(res: StdResult<T, E>) -> Result<()>
where
    E: Into<MyError>,
{
    res.map(|_| ()).map_err(|e| e.into())
}

impl Person {
    pub fn leave_pub(conn: &mut DbConnection, person_id: Uuid) -> Result<()> {
        map_empty(conn.execute(
            "UPDATE person SET last_updated = NOW(), pub_id = NULL WHERE person.id = $1",
            &[&person_id],
        ))
    }

    pub fn leave_table(conn: &mut DbConnection, person_id: Uuid) -> Result<()> {
        map_empty(conn.execute(
            "UPDATE person SET last_updated = NOW(), table_id = NULL WHERE person.id = $1",
            &[&person_id],
        ))
    }

    pub fn load_from_db(conn: &mut DbConnection, person_id: Uuid) -> Result<Person> {
        let rows = conn.query("SELECT * FROM person WHERE person.id = $1", &[&person_id])?;
        let row = rows.get(0);
        Ok(Person {
            id: row.get("id"),
            name: row.get("name"),
            pub_id: row.get("pub_id"),
            table_id: row.get("table_id"),
            last_updated: row.get("last_updated"),
        })
    }

    pub fn add_person(conn: &mut DbConnection, person_id: Uuid) -> Result<()> {
        map_empty(conn.execute(
            "INSERT INTO person (id) VALUES ($1) ON CONFLICT DO NOTHING",
            &[&person_id],
        ))
    }

    pub fn set_name(conn: &mut DbConnection, person_id: Uuid, name: String) -> Result<()> {
        map_empty(conn.execute(
            "UPDATE person SET last_updated = NOW(), name = $2 WHERE person.id = $1",
            &[&person_id, &name],
        ))
    }

    pub fn set_pub(conn: &mut DbConnection, person_id: Uuid, pub_id: Uuid) -> Result<()> {
        map_empty(conn.execute(
            "UPDATE person SET last_updated = NOW(), pub_id = $2 WHERE person.id = $1",
            &[&person_id, &pub_id],
        ))
    }

    pub fn set_table(conn: &mut DbConnection, person_id: Uuid, table_id: Uuid) -> Result<()> {
        map_empty(conn.execute(
            "UPDATE person SET last_updated = NOW(), table_id = $2 WHERE person.id = $1",
            &[&person_id, &table_id],
        ))
    }

    pub fn update_last(conn: &mut DbConnection, person_id: Uuid) -> Result<()> {
        map_empty(conn.execute(
            "UPDATE person SET last_updated = NOW() WHERE person.id = $1",
            &[&person_id],
        ))
    }

    pub fn cleanup_outdated(conn: &mut DbConnection) -> Result<()> {
        map_empty(conn.execute(
            "DELETE FROM person WHERE person.last_updated < (NOW() - interval '5 minutes')",
            &[],
        ))
    }
}

impl Pub {
    pub fn get_pubs(conn: &mut DbConnection) -> Result<Vec<PubWithPeople>> {
        Ok(conn.query("SELECT public_house.*, ARRAY_REMOVE(ARRAY_AGG(person.id), NULL) AS persons FROM public_house LEFT JOIN person ON person.pub_id = public_house.id GROUP BY public_house.id", &[])?
        .iter()
        .map(|row| PubWithPeople {
            id: row.get("id"),
            name: row.get("name"),
            persons: row.get("persons")
        }).collect())
    }

    pub fn add_pub(&self, conn: &mut DbConnection) -> Result<()> {
        map_empty(conn.execute(
            "INSERT INTO public_house (id, name) VALUES ($1, $2)",
            &[&self.id, &self.name],
        ))
    }

    pub fn delete_pub(conn: &mut DbConnection, pub_id: Uuid) -> Result<()> {
        let patrons = conn.query("SELECT id FROM person WHERE person.pub_id = $1", &[&pub_id])?;
        if patrons.is_empty() {
            map_empty(conn.execute("DELETE FROM public_house WHERE id = $1", &[&pub_id]))
        } else {
            Ok(())
        }
    }
}

impl PubTable {
    pub fn get_tables(conn: &mut DbConnection, pub_id: Uuid) -> Result<Vec<TableWithPeople>> {
        Ok(conn.query("SELECT pub_table.*, ARRAY_REMOVE(ARRAY_AGG(person.id), NULL) AS persons FROM pub_table LEFT JOIN person ON person.table_id = pub_table.id WHERE pub_table.pub_id = $1 GROUP BY pub_table.id", &[&pub_id])?
        .iter()
        .map(|row| TableWithPeople {
            id: row.get("id"),
            name: row.get("name"),
            pub_id: row.get("pub_id"),
            persons: row.get("persons")
        }).collect())
    }

    pub fn add_table(&self, conn: &mut DbConnection) -> Result<()> {
        map_empty(conn.execute(
            "INSERT INTO pub_table (id, name, pub_id) VALUES ($1, $2, $3)",
            &[&self.id, &self.name, &self.pub_id],
        ))
    }

    pub fn delete_table(conn: &mut DbConnection, table_id: Uuid) -> Result<Uuid> {
        let patrons = conn.query(
            "SELECT id FROM person WHERE person.table_id = $1",
            &[&table_id],
        )?;
        if patrons.is_empty() {
            let pubs = conn.query(
                "DELETE FROM pub_table WHERE id = $1 RETURNING pub_id",
                &[&table_id],
            )?;
            Ok(pubs.get(0).get("pub_id"))
        } else {
            warn!(
                "Not deleting {table_id} because there's still {} in it",
                patrons.len()
            );
            let pubs = conn.query("SELECT pub_id FROM pub_table WHERE id = $1", &[&table_id])?;
            Ok(pubs.get(0).get("pub_id"))
        }
    }
}
