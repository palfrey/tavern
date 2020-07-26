CREATE TABLE "public_house" (
    id uuid PRIMARY KEY,
    name VARCHAR NOT NULL
);

CREATE TABLE "table" (
    id uuid PRIMARY KEY,
    name VARCHAR NOT NULL,
    pub_id uuid NOT NULL,
    CONSTRAINT fk_table_pub
      FOREIGN KEY(pub_id) 
	  REFERENCES pub(id)
);

CREATE TABLE "person" (
    id uuid PRIMARY KEY,
    name VARCHAR NULL,
    pub_id uuid NULL,
    table_id uuid NULL,
    CONSTRAINT fk_person_pub
      FOREIGN KEY(pub_id) 
	  REFERENCES pub(id),
    CONSTRAINT fk_person_table
      FOREIGN KEY(table_id)
	  REFERENCES pub(id)
);