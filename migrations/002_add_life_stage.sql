CREATE TYPE life_stage AS ENUM ('young', 'has_kids', 'older');

ALTER TABLE customer
    ADD COLUMN life_stage life_stage;
