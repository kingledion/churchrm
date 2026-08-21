-- Soft-delete + audit timestamps for core entities.

ALTER TABLE person
    ADD COLUMN created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN deleted_at TIMESTAMPTZ;

ALTER TABLE family
    ADD COLUMN created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN deleted_at TIMESTAMPTZ;

ALTER TABLE contact
    ADD COLUMN created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN modified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN deleted_at TIMESTAMPTZ;

CREATE INDEX person_active_idx ON person (id) WHERE deleted_at IS NULL;
CREATE INDEX family_active_idx ON family (id) WHERE deleted_at IS NULL;
CREATE INDEX contact_active_idx ON contact (id) WHERE deleted_at IS NULL;
CREATE INDEX person_family_active_idx ON person (family_id) WHERE deleted_at IS NULL;
