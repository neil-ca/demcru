-- Add migration script here
CREATE TABLE contacts(
    id uuid NOT NULL,
    PRIMARY KEY(id),
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    phone SMALLINT UNIQUE,
    created_at timestamptz NOT NULL
);
