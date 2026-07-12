CREATE TABLE customer (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    phone TEXT NOT NULL DEFAULT '',
    email TEXT NOT NULL DEFAULT ''
);

CREATE INDEX customer_name_idx ON customer (name);
