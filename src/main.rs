
use teloxide::Bot;
use teloxide::types::Message;
use teloxide::utils::command::BotCommands;
use teloxide::{prelude::*};
use std::result::Result;
use std::env;
use dotenv::dotenv;


mod core;
mod models;
mod controllers;

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
    EndGame
}



async fn answer(
    bot: Bot,
    message: Message,
    command: Command,
) -> ResponseResult<()> {
    match command {
        Command::Help => { bot.send_message(message.chat.id, Command::descriptions().to_string()).await? },
        Command::NewGame => { bot.send_message(message.chat.id, Command::descriptions().to_string()).await? },
        Command::EndGame => { bot.send_message(message.chat.id, Command::descriptions().to_string()).await? },
    };
    Ok(())
}
