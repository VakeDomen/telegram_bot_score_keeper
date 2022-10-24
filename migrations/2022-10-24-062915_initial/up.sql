-- Your SQL goes here-- Your SQL goes here-- Your SQL goes here
create table users
(
    id              varchar not null primary key,
    name            varchar not null unique,
    chat_id         varchar not null
);

create table chats 
(
    telegram_id     varchar not null,
    default_game    varchar not null
);

create table rounds 
(
    id              varchar not null primary key,
    chat_id         varchar not null,
    round_id        varchar not null,
    game_id         varchar not null,
    player_id       varchar not null,
    tags            varchar not null
);
