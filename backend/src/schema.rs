use diesel::{allow_tables_to_appear_in_same_query, joinable, table};

table! {
    person (id) {
        id -> Uuid,
        name -> Nullable<Varchar>,
        pub_id -> Nullable<Uuid>,
        table_id -> Nullable<Uuid>,
    }
}

table! {
    public_house (id) {
        id -> Uuid,
        name -> Varchar,
    }
}

table! {
    pub_table (id) {
        id -> Uuid,
        name -> Varchar,
        pub_id -> Uuid,
    }
}

joinable!(pub_table -> public_house (pub_id));

allow_tables_to_appear_in_same_query!(person, public_house, pub_table);
