CREATE TABLE "pub_table" (
    id UUID PRIMARY KEY,
    name VARCHAR NOT NULL,
    pub_id UUID NOT NULL,
    CONSTRAINT fk_table_pub FOREIGN KEY(pub_id) REFERENCES public_house(id)
);
