-- Your SQL goes here
CREATE TABLE Users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    hash VARCHAR(1000) NOT NULL
);