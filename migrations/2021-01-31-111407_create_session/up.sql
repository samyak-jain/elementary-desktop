-- Your SQL goes here
CREATE TABLE matrix_session (
    user_id TEXT NOT NULL PRIMARY KEY,
    access_token TEXT NOT NULL,
    device_id TEXT NOT NULL
)
