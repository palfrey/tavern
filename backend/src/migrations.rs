use postgres::Client;
use postgres::Transaction;
use schemamama::{migration, Migrator};
use schemamama_postgres::{PostgresAdapter, PostgresMigration};

struct CreateAll;
migration!(CreateAll, 1, "create all the tables");

impl PostgresMigration for CreateAll {
    fn up(&self, transaction: &mut Transaction) -> Result<(), postgres::Error> {
        transaction
            .execute(
                r#"
        CREATE TABLE "public_house" (
            id uuid PRIMARY KEY,
            name VARCHAR NOT NULL
        );"#,
                &[],
            )
            .and_then(|_| {
                transaction
                    .execute(
                        r#"
        CREATE TABLE "pub_table" (
            id uuid PRIMARY KEY,
            name VARCHAR NOT NULL,
            pub_id uuid NOT NULL,
            CONSTRAINT fk_table_pub
              FOREIGN KEY(pub_id)
              REFERENCES public_house(id)
        );"#,
                        &[],
                    )
                    .and_then(|_| {
                        transaction.execute(
                            r#"
        CREATE TABLE "person" (
            id uuid PRIMARY KEY,
            name VARCHAR NULL,
            pub_id uuid NULL,
            table_id uuid NULL,
            CONSTRAINT fk_person_pub
              FOREIGN KEY(pub_id)
              REFERENCES public_house(id),
            CONSTRAINT fk_person_table
              FOREIGN KEY(table_id)
              REFERENCES pub_table(id)
        );"#,
                            &[],
                        )
                    })
            })
            .map(|_| ())
    }

    fn down(&self, transaction: &mut Transaction) -> Result<(), postgres::Error> {
        transaction
            .execute(
                r#"DROP TABLE "person"
        DROP TABLE "table"
        DROP TABLE "pub"
        "#,
                &[],
            )
            .map(|_| ())
    }
}

struct AddLastUpdated;
migration!(AddLastUpdated, 2, "last_updated in person");

impl PostgresMigration for AddLastUpdated {
    fn up(&self, transaction: &mut Transaction) -> Result<(), postgres::Error> {
        transaction
            .execute(
                "ALTER TABLE person ADD COLUMN last_updated TIMESTAMP NOT NULL DEFAULT now()",
                &[],
            )
            .map(|_| ())
    }

    fn down(&self, transaction: &mut Transaction) -> Result<(), postgres::Error> {
        transaction
            .execute("ALTER TABLE person DROP column last_updated", &[])
            .map(|_| ())
    }
}

pub fn run_migrations(client: &mut Client) {
    let adapter = PostgresAdapter::new(client);
    // Create the metadata tables necessary for tracking migrations. This is safe to call more than
    // once (`CREATE TABLE IF NOT EXISTS schemamama` is used internally):
    adapter.setup_schema().unwrap();

    let mut migrator = Migrator::new(adapter);

    migrator.register(Box::new(CreateAll));
    migrator.register(Box::new(AddLastUpdated));
    migrator.up(None).unwrap();
}
