use crate::error::{MyError, Result};
use crate::types::{DbConnection, Person, Pool, Pub, PubTable, PubWithPeople, TableWithPeople};
use bb8_postgres::PostgresConnectionManager;
use log::warn;
use postgres::NoTls;
use std::env;
use std::result::Result as StdResult;
use uuid::Uuid;

pub fn get_db_url() -> String {
    env::var("DATABASE_URL").expect("Database url not set")
}

pub async fn make_pool() -> Pool {
    let manager = PostgresConnectionManager::new(get_db_url().parse().unwrap(), NoTls);
    Pool::builder()
        .build(manager)
        .await
        .expect("Failed to create db pool")
}

fn map_empty<T, E>(res: StdResult<T, E>) -> Result<()>
where
    E: Into<MyError>,
{
    res.map(|_| ()).map_err(|e| e.into())
}

impl Person {
    pub async fn leave_pub<'a>(conn: &mut DbConnection<'a>, person_id: Uuid) -> Result<()> {
        map_empty(
            conn.execute(
                "UPDATE person SET last_updated = NOW(), pub_id = NULL WHERE person.id = $1",
                &[&person_id],
            )
            .await,
        )
    }

    pub async fn leave_table<'a>(conn: &mut DbConnection<'a>, person_id: Uuid) -> Result<()> {
        map_empty(
            conn.execute(
                "UPDATE person SET last_updated = NOW(), table_id = NULL WHERE person.id = $1",
                &[&person_id],
            )
            .await,
        )
    }

    pub async fn load_from_db<'a>(conn: &mut DbConnection<'a>, person_id: Uuid) -> Result<Person> {
        let rows = conn
            .query("SELECT * FROM person WHERE person.id = $1", &[&person_id])
            .await?;
        let row = rows.get(0).unwrap();
        Ok(Person {
            id: row.get("id"),
            name: row.get("name"),
            pub_id: row.get("pub_id"),
            table_id: row.get("table_id"),
            last_updated: row.get("last_updated"),
        })
    }

    pub async fn add_person<'a>(conn: &DbConnection<'a>, person_id: Uuid) -> Result<()> {
        map_empty(
            conn.execute(
                "INSERT INTO person (id) VALUES ($1) ON CONFLICT DO NOTHING",
                &[&person_id],
            )
            .await,
        )
    }

    pub async fn set_name<'a>(
        conn: &mut DbConnection<'a>,
        person_id: Uuid,
        name: String,
    ) -> Result<()> {
        map_empty(
            conn.execute(
                "UPDATE person SET last_updated = NOW(), name = $2 WHERE person.id = $1",
                &[&person_id, &name],
            )
            .await,
        )
    }

    pub async fn set_pub<'a>(
        conn: &mut DbConnection<'a>,
        person_id: Uuid,
        pub_id: Uuid,
    ) -> Result<()> {
        map_empty(
            conn.execute(
                "UPDATE person SET last_updated = NOW(), pub_id = $2 WHERE person.id = $1",
                &[&person_id, &pub_id],
            )
            .await,
        )
    }

    pub async fn set_table<'a>(
        conn: &mut DbConnection<'a>,
        person_id: Uuid,
        table_id: Uuid,
    ) -> Result<()> {
        map_empty(
            conn.execute(
                "UPDATE person SET last_updated = NOW(), table_id = $2 WHERE person.id = $1",
                &[&person_id, &table_id],
            )
            .await,
        )
    }

    pub async fn update_last<'a>(conn: &mut DbConnection<'a>, person_id: Uuid) -> Result<()> {
        map_empty(
            conn.execute(
                "UPDATE person SET last_updated = NOW() WHERE person.id = $1",
                &[&person_id],
            )
            .await,
        )
    }

    pub async fn cleanup_outdated<'a>(conn: &mut DbConnection<'a>) -> Result<()> {
        map_empty(
            conn.execute(
                "DELETE FROM person WHERE person.last_updated < (NOW() - interval '5 minutes')",
                &[],
            )
            .await,
        )
    }
}

impl Pub {
    pub async fn get_pubs<'a>(conn: &mut DbConnection<'a>) -> Result<Vec<PubWithPeople>> {
        Ok(conn.query("SELECT public_house.*, ARRAY_REMOVE(ARRAY_AGG(person.id), NULL) AS persons FROM public_house LEFT JOIN person ON person.pub_id = public_house.id GROUP BY public_house.id", &[]).await?
        .iter()
        .map(|row| PubWithPeople {
            id: row.get("id"),
            name: row.get("name"),
            persons: row.get("persons")
        }).collect())
    }

    pub async fn add_pub<'a>(&self, conn: &mut DbConnection<'a>) -> Result<()> {
        map_empty(
            conn.execute(
                "INSERT INTO public_house (id, name) VALUES ($1, $2)",
                &[&self.id, &self.name],
            )
            .await,
        )
    }

    pub async fn delete_pub<'a>(conn: &mut DbConnection<'a>, pub_id: Uuid) -> Result<()> {
        let patrons = conn
            .query("SELECT id FROM person WHERE person.pub_id = $1", &[&pub_id])
            .await?;
        if patrons.is_empty() {
            map_empty(
                conn.execute("DELETE FROM public_house WHERE id = $1", &[&pub_id])
                    .await,
            )
        } else {
            Ok(())
        }
    }
}

impl PubTable {
    pub async fn get_tables<'a>(
        conn: &mut DbConnection<'a>,
        pub_id: Uuid,
    ) -> Result<Vec<TableWithPeople>> {
        Ok(conn.query("SELECT pub_table.*, ARRAY_REMOVE(ARRAY_AGG(person.id), NULL) AS persons FROM pub_table LEFT JOIN person ON person.table_id = pub_table.id WHERE pub_table.pub_id = $1 GROUP BY pub_table.id", &[&pub_id]).await?
        .iter()
        .map(|row| TableWithPeople {
            id: row.get("id"),
            name: row.get("name"),
            pub_id: row.get("pub_id"),
            persons: row.get("persons")
        }).collect())
    }

    pub async fn add_table<'a>(&self, conn: &mut DbConnection<'a>) -> Result<()> {
        map_empty(
            conn.execute(
                "INSERT INTO pub_table (id, name, pub_id) VALUES ($1, $2, $3)",
                &[&self.id, &self.name, &self.pub_id],
            )
            .await,
        )
    }

    pub async fn delete_table<'a>(conn: &mut DbConnection<'a>, table_id: Uuid) -> Result<Uuid> {
        let patrons = conn
            .query(
                "SELECT id FROM person WHERE person.table_id = $1",
                &[&table_id],
            )
            .await?;
        if patrons.is_empty() {
            let pubs = conn
                .query(
                    "DELETE FROM pub_table WHERE id = $1 RETURNING pub_id",
                    &[&table_id],
                )
                .await?;
            Ok(pubs.get(0).unwrap().get("pub_id"))
        } else {
            warn!(
                "Not deleting {table_id} because there's still {} in it",
                patrons.len()
            );
            let pubs = conn
                .query("SELECT pub_id FROM pub_table WHERE id = $1", &[&table_id])
                .await?;
            Ok(pubs.get(0).unwrap().get("pub_id"))
        }
    }
}
