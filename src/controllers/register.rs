use teloxide::{Bot, types::Message};

use crate::{models::user::{NewUser, User}, core::{game_aggregator::GameAggregator, message_helper::extract_message_text}};
use crate::core::database::user_operations::insert_user;

pub fn register(
    _: &Bot,
    message: Message,
) -> String {
    let chat_id = message.chat.id;
    
    let text = match extract_message_text(&message) {
        Some(text) => text,
        None => return "Enter names of users".to_string()
    };
    // extract amount to be loaned to recievers
    let names = extract_names(text.as_str());
    let mut valid_new_users = vec![];
    let mut invalid_new_users = vec![];

    for user in names
        .iter()
        .map(|n| NewUser::from(n.to_uppercase(), chat_id.to_string()))
        .map(|mut u| { GameAggregator::new().validate_user(&mut u); u }) 
    {
        if user.is_valid() {
            valid_new_users.push(user);
        } else {
            invalid_new_users.push(user);
        }
    }

    let validate_messages: Vec<String> = invalid_new_users.into_iter()
        .map(|u| format!("Username {} is on a reserved list. Choose another name.", u.name))
        .collect();

    let insert_messages: Vec<String> = valid_new_users
        .into_iter()
        .filter_map(|u| User::from(u).ok())
        .map(|u| match insert_user(u){
            Ok(u) => format!("User {} created!", u.name),
            Err(e) => format!("Something ent wrong crating user: {}", e),
        })
        .collect();
    
    let part_one = insert_messages.join("\n");
    let part_two = validate_messages.join("\n");
    format!("{}\n{}", part_one, part_two)

}


fn extract_names(text: &str) -> Vec<&str> {
    text.split_whitespace().skip(1).collect::<Vec<&str>>()
}