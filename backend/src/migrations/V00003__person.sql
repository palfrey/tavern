CREATE TABLE "person" (
    id UUID PRIMARY KEY,
    name VARCHAR NULL,
    pub_id UUID NULL,
    table_id UUID NULL,
    last_updated TIMESTAMP NOT NULL DEFAULT now(),
    CONSTRAINT fk_person_pub
    FOREIGN KEY(pub_id)
    REFERENCES public_house(id),
    CONSTRAINT fk_person_table
    FOREIGN KEY(table_id)
    REFERENCES pub_table(id)
);
