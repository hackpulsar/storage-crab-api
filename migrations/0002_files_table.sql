create table files(
    id serial primary key,
    filename varchar not null,
    path varchar not null,
    size bigint not null,
    uploaded_at timestamp default now(),
    user_id integer not null,
    constraint fk_user
        foreign key(user_id)
        references users(id)
        on delete cascade
)