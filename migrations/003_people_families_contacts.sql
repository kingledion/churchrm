-- Replace flat customer contacts with person / family / contact.

CREATE TYPE life_stage_new AS ENUM ('child', 'young_adult', 'parent', 'older');

CREATE TABLE family (
    id UUID PRIMARY KEY
);

CREATE TABLE person (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL DEFAULT '',
    phone TEXT NOT NULL DEFAULT '',
    email TEXT NOT NULL DEFAULT '',
    life_stage life_stage_new,
    family_id UUID REFERENCES family (id) ON DELETE CASCADE
);

CREATE TABLE contact (
    id UUID PRIMARY KEY,
    person_id UUID REFERENCES person (id) ON DELETE CASCADE,
    family_id UUID REFERENCES family (id) ON DELETE CASCADE,
    CHECK (
        (person_id IS NOT NULL AND family_id IS NULL)
        OR (person_id IS NULL AND family_id IS NOT NULL)
    )
);

INSERT INTO person (id, name, phone, email, life_stage, family_id)
SELECT
    id,
    name,
    phone,
    email,
    CASE life_stage::text
        WHEN 'young' THEN 'young_adult'::life_stage_new
        WHEN 'has_kids' THEN 'parent'::life_stage_new
        WHEN 'older' THEN 'older'::life_stage_new
        ELSE NULL
    END,
    NULL
FROM customer;

INSERT INTO contact (id, person_id, family_id)
SELECT id, id, NULL
FROM customer;

DROP TABLE customer;
DROP TYPE life_stage;
ALTER TYPE life_stage_new RENAME TO life_stage;

CREATE INDEX person_name_idx ON person (name);
CREATE INDEX person_family_id_idx ON person (family_id);
CREATE INDEX contact_person_id_idx ON contact (person_id);
CREATE INDEX contact_family_id_idx ON contact (family_id);
