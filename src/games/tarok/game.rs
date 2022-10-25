use std::{collections::HashMap, io::{Error, ErrorKind}};
use chrono::Utc;

use crate::{core::{traits::{CheckName, Game}, message_helper::extract_message_text, database::user_operations::get_user_by_name}, models::user::User};

use super::enums::{TarokGameInput, TarokGame, TarokGameAttribute, TarokPlayerAttibute, TarokPlayerInput};

pub struct Tarok {
    players: Vec<User>,
    _radlci: HashMap<String, i32>,
    score: HashMap<String, Vec<Option<i32>>>,
    round: i32,
}

impl Tarok {
    pub fn new() -> Self {
        Self {
            players: vec![],
            _radlci: HashMap::new(),
            score: HashMap::new(),
            round: 0
        }
    }
}

impl CheckName for Tarok {
    fn get_reserved_terms(&self) -> &'static [&'static str] {
        &["3","2", "1", "S3", "S2", "S1"]
    }
}


impl Game for Tarok {
    fn start_game(&mut self) -> Result<String, std::io::Error> {
        Ok("Started game of Tarok!".to_string())
    }

    fn handle_round(&mut self, message: teloxide::types::Message) -> Result<String, std::io::Error> {
        let text = match extract_message_text(&message) {
            Some(text) => text,
            None => return Err(Error::new(ErrorKind::Other, "Failed to extract message text".to_string()))
        };

        let users = match extract_round_users(&text) {
            Ok(users) => users,
            Err(e) => return Err(e),
        };

        handle_new_users(
            &users, 
            &mut self.players, 
            &mut self.score, 
            &self.round
        );
        
        let game_attributes: Vec<TarokGameInput> = match extract_game_attributes(&text) {
            Ok(attr) => attr,
            Err(e) => return Err(Error::new(ErrorKind::Other, format!("Failed to extract game attributes: {}", e)))
        };

        let player_attributes: HashMap<String, Vec<TarokPlayerInput>> = match extract_player_attributes(&self.players, &text) {
            Ok(attr) => attr,
            Err(e) => return Err(Error::new(ErrorKind::Other, format!("Failed to extract player attributes: {}", e)))
        };

        self.round += 1;
        Ok(format!("{:#?} \n{:#?}\n{:#?}", extract_round_game_fragment(&text), game_attributes, player_attributes))
    }

    fn end_game(self: Box<Self>) -> Result<String, std::io::Error> {
        todo!()
    }

    fn get_state(&mut self) -> Result<String, std::io::Error> {
        todo!()
    }

    fn generate_file_name(&self) -> String { format!("{}_tarok.html", Utc::now()) }
}

fn extract_player_attributes(players: &[User], text: &String) -> Result<HashMap<String, Vec<TarokPlayerInput>>, Error> {
    let fragment = match extract_round_player_fragment(&text) {
        Some(frag) => frag,
        None => return Err(Error::new(ErrorKind::Other, "Failed to locate player fragment".to_string())),
    };
    let mut out = HashMap::new();
    for player_fragment in fragment.split(' ') {
        let mut inputs = vec![];
        let user = match parse_user_from_fragment(&player_fragment.to_string()) {
            Ok(user) => user,
            Err(e) => return Err(Error::new(ErrorKind::Other, format!("Failed parsing player from fragment: {}", e))),
        };
        for player_partial in player_fragment.split(',').skip(1) {
            let mut attr_option = match parse_player_attribute_fragment(player_partial) {
                Some(val) => Some(TarokPlayerInput::PlayerAttribute(val)),
                None => None,
            };
            let mut diff_option = match parse_diff_option_fragment(player_partial) {
                Some(val) => Some(TarokPlayerInput::PlayerDiff(val)),
                None => None,
            };
            match (attr_option, diff_option) {
                (None, None) => return Err(Error::new(ErrorKind::Other, format!("Could not recognize player attribute: {}", player_partial))),
                (_, Some(diff)) => inputs.push(diff),
                (Some(attr), _) => inputs.push(attr),
            }
        }
        out.insert(user.id, inputs);
    }
    Ok(out)
}

fn extract_game_attributes(text: &String) -> Result<Vec<TarokGameInput>, Error> {
    let fragment = match extract_round_game_fragment(text) {
        Some(fr) => fr,
        None => return Err(Error::new(ErrorKind::Other, "Failed to locate game fragment".to_string())),
    };
    let mut inputs = vec![];
    let mut game_found = false; // only one fragment can be a game input
    let mut game_diff_found = false; // only one fragment can be a game diff input
    for partial_fragment in fragment.split(',') {
        let mut game_option = match parse_game_option_fragment(partial_fragment) {
            Some(val) => Some(TarokGameInput::TarokGame(val)),
            None => None,
        };
        let attribute_option = match parse_attribute_option_fragment(partial_fragment) {
            Some(val) => Some(TarokGameInput::TarokGameAttribute(val)),
            None => None,
        };
        let mut diff_option = match parse_diff_option_fragment(partial_fragment) {
            Some(val) => Some(TarokGameInput::TarokGameDiff(val)),
            None => None,
        };
        // just to make sure only one game can be defined
        // allows us to have attibutes with same name after the game 
        // has beed specified
        if game_found {
            game_option = None;
        }
        if game_option.is_some() {
            game_found = true;
        }
        // just to make sure only one game diff can be defined
        if game_diff_found {
            diff_option = None;
        }
        if diff_option.is_some() {
            game_diff_found = true;
        }
        match (game_option, attribute_option, diff_option) {
            (None, None, None) => return Err(Error::new(ErrorKind::Other, format!("Could not recognize game attribute: {}", partial_fragment))),
            (Some(val), _, _) => inputs.push(val),
            (_, Some(val), _) => inputs.push(val),
            (_, _, Some(val)) => inputs.push(val),
        };
    }

    match game_found {
        true => Ok(inputs),
        false => return Err(Error::new(ErrorKind::Other, format!("No game specified."))),
    }
    
}

fn parse_player_attribute_fragment(partial_fragment: &str) -> Option<TarokPlayerAttibute> {
    match partial_fragment.to_uppercase().as_str() {
        "M" => Some(TarokPlayerAttibute::M),
        "R" => Some(TarokPlayerAttibute::R),
        "T" => Some(TarokPlayerAttibute::T),
        _ => None,
    }
}

fn parse_diff_option_fragment(partial_fragment: &str) -> Option<i32> {
    match partial_fragment.parse() {
        Ok(val) => Some(val),
        Err(_) => None,
    }
}

fn parse_attribute_option_fragment(partial_fragment: &str) -> Option<TarokGameAttribute> {
    match partial_fragment.to_uppercase().as_str() {
        "P" => Some(TarokGameAttribute::P),
        "K" => Some(TarokGameAttribute::K),
        "V" => Some(TarokGameAttribute::V),
        "T" => Some(TarokGameAttribute::T),
        "NP" => Some(TarokGameAttribute::NP),
        "NK" => Some(TarokGameAttribute::NK),
        "NV" => Some(TarokGameAttribute::NV),
        "NT" => Some(TarokGameAttribute::NT),
        _ => None
    }
}

fn parse_game_option_fragment(partial_fragment: &str) -> Option<TarokGame> {
    match partial_fragment.to_uppercase().as_str() {
        "I3" => Some(TarokGame::I3),
        "I2" => Some(TarokGame::I2),
        "I1" => Some(TarokGame::I1),
        "S3" => Some(TarokGame::S3),
        "S2" => Some(TarokGame::S2),
        "S1" => Some(TarokGame::S1),
        "SB" => Some(TarokGame::SB),
        "KL" => Some(TarokGame::KL),
        "B" => Some(TarokGame::B),
        "P" => Some(TarokGame::P),
        "BVI3" => Some(TarokGame::BVI3),
        "BVI2" => Some(TarokGame::BVI2),
        "BVI1" => Some(TarokGame::BVI1),
        "BVS3" => Some(TarokGame::BVS3),
        "BVS2" => Some(TarokGame::BVS2),
        "BVS1" => Some(TarokGame::BVS1),
        "BVSB" => Some(TarokGame::BVSB),
        _ => None
    }
}

fn handle_new_users(
    users: &Vec<User>, 
    players: &mut Vec<User>, 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    round: &i32
) {
    for user in users.iter() {
        if !players.contains(user) {
            players.push(user.clone());
            let mut player_score = vec![];
            fill_gaps_until_round(&mut player_score, round);
            score.insert(user.id.clone(), player_score);
        }
    }
}

fn fill_gaps_until_round(score: &mut Vec<Option<i32>>, round: &i32) {
    if score.len() < (*round) as usize {
        for _ in score.len()..(*round) as usize {
            score.push(None);
        }
    }
}

fn extract_round_users(message_text: &String) -> Result<Vec<User>, Error> {
    let fragment = match extract_round_player_fragment(message_text) {
        Some(fragment) => fragment,
        None => return Err(Error::new(ErrorKind::Other, "Can't find any users to parse :(".to_string())),
    };
    match parse_users_from_fragment(&fragment) {
        Ok(users) => Ok(users),
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Failed parsing players: {}", e))),
    }
}

fn parse_users_from_fragment(fragment: &String) -> Result<Vec<User>, Error> {
    let mut users = vec![];
    for user_framgent in fragment.split(' ') {
        // extract name from user fragment (JAN,M -> JAN)
        match parse_user_from_fragment(&user_framgent.to_string()) {
            Ok(user) => users.push(user),
            Err(e) => return Err(Error::new(ErrorKind::Other, format!("Failed parsing player: {}", e))),
        };
    }
    Ok(users)
}

fn parse_user_from_fragment(fragment: &String) -> Result<User, Error> {
    let user_name = match fragment.split(',').nth(0) {
        Some(name) => name,
        None => return Err(Error::new(ErrorKind::Other, format!("Can't parse a users :( {}", fragment))),
    };
    // try to find user in database
    let user_option = match get_user_by_name(user_name.to_uppercase()) {
        Ok(data) => data,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Error fetching user from DB: {}", e))),
    };
    // check if user found in database
    match user_option {
        Some(user) => Ok(user),
        None => return Err(Error::new(ErrorKind::Other, "Error fetching user from DB".to_string())),
    }
}

fn extract_round_game_fragment(message_text: &String) -> Option<String> {
    let fragment = message_text
        .split(' ')
        .nth(1);
    match fragment {
        Some(fr) => Some(fr.to_string()),
        None => None,
    }
}

fn extract_round_player_fragment(message_text: &String) -> Option<String> {
    let fragments: Vec<&str> = message_text
        .split(' ')
        .skip(2)
        .collect();
    if fragments.is_empty() {
        None
    } else {
        Some(fragments.join(" "))
    }
}