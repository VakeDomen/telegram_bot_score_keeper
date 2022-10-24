
use controllers::end_game::end_game;
use controllers::new_game::new_game;
use controllers::register::register;
use controllers::score_round::score_round;
use teloxide::Bot;
use teloxide::types::Message;
use teloxide::utils::command::BotCommands;
use teloxide::prelude::*;
use std::result::Result;
use std::env;
use dotenv::dotenv;


mod core;
mod models;
mod controllers;
mod games;


#[macro_use] extern crate diesel;
extern crate serde_json;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("TELEGRAM_BOT_TOKEN").expect("$TELEGRAM_BOT_TOKEN is not set");
    env::set_var("TELOXIDE_TOKEN", token);
    pretty_env_logger::init();
    let bot = Bot::from_env();
    println!("Running telegram bot!");
    teloxide::commands_repl(bot, answer, Command::ty()).await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "Start new game")]
    NewGame,
    #[command(description = "Unsubscribe from jobs")]
    EndGame,
    #[command(description = "Register new players")]
    Register,
    #[command(description = "Round of game")]
    Score,
}



async fn answer(
    bot: Bot,
    message: Message,
    command: Command,
) -> ResponseResult<()> {
    match command {
        Command::Help => { bot.send_message(message.chat.id, Command::descriptions().to_string()).await?; },
        Command::Register => { bot.send_message(message.chat.id, register(&bot, message)).await?; },
        Command::NewGame => { bot.send_message(message.chat.id, new_game(&bot, message).await).await?; },
        Command::EndGame => end_game_handler(bot, message).await,
        Command::Score => { bot.send_message(message.chat.id, score_round(&bot, message).await).await?; },
    };
    Ok(())
}

async fn end_game_handler(bot: Bot, message: Message) -> () {
    let id = message.chat.id.clone();
    match end_game(&bot, message).await {
        Ok(file) => { let _ = bot.send_document(id, file).await; },
        Err(e) => {let _ = bot.send_message(id, e.to_string()).await;},
    };
}

