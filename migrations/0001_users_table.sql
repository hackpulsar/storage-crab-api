create table users (
    id serial primary key,
    email varchar not null,
    username varchar not null,
    password_hash varchar not null,
    salt varchar not null
)