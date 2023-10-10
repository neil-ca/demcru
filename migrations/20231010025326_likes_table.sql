CREATE TABLE likes(
    id uuid PRIMARY KEY,
    user_id TEXT NOT NULL,
    counter INTEGER NOT NULL
);
