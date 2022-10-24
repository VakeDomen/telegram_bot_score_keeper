table! {
    rounds (id) {
        id -> Text,
        chat_id -> Text,
        round_id -> Text,
        game_id -> Text,
        player_id -> Text,
        tags -> Text,
    }
}

table! {
    users (id) {
        id -> Text,
        name -> Text,
        chat_id -> Text,
    }
}

table! {
    chats (id) {
        id -> Text,
        telegram_id -> Text,
        default_game -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    rounds,
    chats,
    users,
);
