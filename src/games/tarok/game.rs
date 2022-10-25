use std::{collections::HashMap, io::{Error, ErrorKind}};
use chrono::Utc;

use crate::{core::{traits::{CheckName, Game}, message_helper::extract_message_text, database::user_operations::get_user_by_name}, models::user::User};

use super::enums::{TarokGameInput, TarokGame, TarokGameAttribute, TarokPlayerAttibute, TarokPlayerInput};

pub struct Tarok {
    players: Vec<User>,
    radlci: HashMap<String, i32>,
    score: HashMap<String, Vec<Option<i32>>>,
    player_attributes: HashMap<String, Vec<Option<TarokPlayerInput>>>,
    game_attributes: HashMap<String, Vec<Option<TarokGameInput>>>,
    round: i32,
}

impl Tarok {
    pub fn new() -> Self {
        Self {
            players: vec![],
            radlci: HashMap::new(),
            score: HashMap::new(),
            player_attributes: HashMap::new(),
            game_attributes: HashMap::new(),
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

        match handle_game(
            &mut self.players, 
            &mut self.score, 
            &self.round,
            &mut radlci,
            &mut self.player_attributes,
            &mut self.game_attributes,
            player_attributes,
            game_attributes,
        ) {
            Ok(_) => (),
            Err(e) => return Err(Error::new(ErrorKind::Other, format!("Failed to calculate round: {}", e)))
        }

        self.round += 1;
        Ok(format!("{:#?} \n", extract_round_game_fragment(&text)))
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

fn handle_game(
    players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    round: &i32, 
    radlci: HashMap<String, i32>,
    global_player_attributes: &mut HashMap<String, Vec<Option<TarokPlayerInput>>>,
    global_game_attributes: &mut HashMap<String, Vec<Option<TarokGameInput>>>,
    round_player_attributes: HashMap<String, Vec<TarokPlayerInput>>,
    round_game_attributes: Vec<TarokGameInput>,
) -> Result<(), Error> {
    // find what game we are playing
    let game: TarokGame = match find_tarok_game(&round_game_attributes) {
        Some(game) => game,
        None => return Err(Error::new(ErrorKind::Other, "No game specified".to_string())),
    };
    match game {
        TarokGame::I3 => playI3(players, score, round, radlci, global_player_attributes, global_game_attributes, round_player_attributes, round_game_attributes),
        TarokGame::I2 => playI2(players, score, round, radlci, global_player_attributes, global_game_attributes, round_player_attributes, round_game_attributes),
        TarokGame::I1 => playI1(players, score, round, radlci, global_player_attributes, global_game_attributes, round_player_attributes, round_game_attributes),
        TarokGame::S3 => playS3(players, score, round, radlci, global_player_attributes, global_game_attributes, round_player_attributes, round_game_attributes),
        TarokGame::S2 => playS2(players, score, round, radlci, global_player_attributes, global_game_attributes, round_player_attributes, round_game_attributes),
        TarokGame::S1 => playS1(players, score, round, radlci, global_player_attributes, global_game_attributes, round_player_attributes, round_game_attributes),
        TarokGame::SB => playSB(players, score, round, radlci, global_player_attributes, global_game_attributes, round_player_attributes, round_game_attributes),
        TarokGame::KL => playKL(players, score, round, radlci, global_player_attributes, global_game_attributes, round_player_attributes, round_game_attributes),
        TarokGame::B => playB(players, score, round, radlci, global_player_attributes, global_game_attributes, round_player_attributes, round_game_attributes),
        TarokGame::P => playP(players, score, round, radlci, global_player_attributes, global_game_attributes, round_player_attributes, round_game_attributes),
        TarokGame::BVI3 => playBVI3(players, score, round, radlci, global_player_attributes, global_game_attributes, round_player_attributes, round_game_attributes),
        TarokGame::BVI2 => playBVI2(players, score, round, radlci, global_player_attributes, global_game_attributes, round_player_attributes, round_game_attributes),
        TarokGame::BVI1 => playBVI1(players, score, round, radlci, global_player_attributes, global_game_attributes, round_player_attributes, round_game_attributes),
        TarokGame::BVS3 => playBVS3(players, score, round, radlci, global_player_attributes, global_game_attributes, round_player_attributes, round_game_attributes),
        TarokGame::BVS2 => playBVS2(players, score, round, radlci, global_player_attributes, global_game_attributes, round_player_attributes, round_game_attributes),
        TarokGame::BVS1 => playBVS1(players, score, round, radlci, global_player_attributes, global_game_attributes, round_player_attributes, round_game_attributes),
        TarokGame::BVSB => playBVSB(players, score, round, radlci, global_player_attributes, global_game_attributes, round_player_attributes, round_game_attributes),
    }
}

fn playBVSB(
    players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    round: &i32, 
    radlci: HashMap<String, i32>,
    global_player_attributes: &mut HashMap<String, Vec<Option<TarokPlayerInput>>>, 
    global_game_attributes: &mut HashMap<String, Vec<Option<TarokGameInput>>>, 
    round_player_attributes: HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: Vec<TarokGameInput>
) -> Result<(), Error> {
    let points = match game_worth(TarokGame::BVSB) {
        Some(p) => p,
        None => return Err(Error::new(ErrorKind::Other, format!("Can't determine points for game TarokGame::BVSB")))
    };
    Ok(())
}

fn playBVS1(
    players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    round: &i32, 
    radlci: HashMap<String, i32>,
    global_player_attributes: &mut HashMap<String, Vec<Option<TarokPlayerInput>>>, 
    global_game_attributes: &mut HashMap<String, Vec<Option<TarokGameInput>>>, 
    round_player_attributes: HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: Vec<TarokGameInput>
) -> Result<(), Error> {
    let points = match game_worth(TarokGame::BVS1) {
        Some(p) => p,
        None => return Err(Error::new(ErrorKind::Other, format!("Can't determine points for game TarokGame::BVS1")))
    };
    Ok(())
}

fn playBVS2(
    players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    round: &i32, 
    radlci: HashMap<String, i32>,
    global_player_attributes: &mut HashMap<String, Vec<Option<TarokPlayerInput>>>, 
    global_game_attributes: &mut HashMap<String, Vec<Option<TarokGameInput>>>, 
    round_player_attributes: HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: Vec<TarokGameInput>
) -> Result<(), Error> {
    let points = match game_worth(TarokGame::BVS2) {
        Some(p) => p,
        None => return Err(Error::new(ErrorKind::Other, format!("Can't determine points for game TarokGame::BVS2")))
    };
    Ok(())
}

fn playBVS3(
    players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    round: &i32, 
    radlci: HashMap<String, i32>,
    global_player_attributes: &mut HashMap<String, Vec<Option<TarokPlayerInput>>>, 
    global_game_attributes: &mut HashMap<String, Vec<Option<TarokGameInput>>>, 
    round_player_attributes: HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: Vec<TarokGameInput>
) -> Result<(), Error> {
    let points = match game_worth(TarokGame::BVS3) {
        Some(p) => p,
        None => return Err(Error::new(ErrorKind::Other, format!("Can't determine points for game TarokGame::BVS3")))
    };
    Ok(())
}

fn playBVI1(
    players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    round: &i32, 
    radlci: HashMap<String, i32>,
    global_player_attributes: &mut HashMap<String, Vec<Option<TarokPlayerInput>>>, 
    global_game_attributes: &mut HashMap<String, Vec<Option<TarokGameInput>>>, 
    round_player_attributes: HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: Vec<TarokGameInput>
) -> Result<(), Error> {
    let points = match game_worth(TarokGame::BVI1) {
        Some(p) => p,
        None => return Err(Error::new(ErrorKind::Other, format!("Can't determine points for game TarokGame::BVI1")))
    };
    Ok(())
}

fn playBVI2(
    players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    round: &i32, 
    radlci: HashMap<String, i32>,
    global_player_attributes: &mut HashMap<String, Vec<Option<TarokPlayerInput>>>, 
    global_game_attributes: &mut HashMap<String, Vec<Option<TarokGameInput>>>, 
    round_player_attributes: HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: Vec<TarokGameInput>
) -> Result<(), Error> {
    let points = match game_worth(TarokGame::BVI2) {
        Some(p) => p,
        None => return Err(Error::new(ErrorKind::Other, format!("Can't determine points for game TarokGame::BVI2")))
    };
    Ok(())
}

fn playBVI3(
    players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    round: &i32, 
    radlci: HashMap<String, i32>,
    global_player_attributes: &mut HashMap<String, Vec<Option<TarokPlayerInput>>>, 
    global_game_attributes: &mut HashMap<String, Vec<Option<TarokGameInput>>>, 
    round_player_attributes: HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: Vec<TarokGameInput>
) -> Result<(), Error> {
    let points = match game_worth(TarokGame::BVI3) {
        Some(p) => p,
        None => return Err(Error::new(ErrorKind::Other, format!("Can't determine points for game TarokGame::BVI3")))
    };
    Ok(())
}

fn playP(
    players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    round: &i32, 
    radlci: HashMap<String, i32>,
    global_player_attributes: &mut HashMap<String, Vec<Option<TarokPlayerInput>>>, 
    global_game_attributes: &mut HashMap<String, Vec<Option<TarokGameInput>>>, 
    round_player_attributes: HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: Vec<TarokGameInput>
) -> Result<(), Error> {
    let points = match game_worth(TarokGame::P) {
        Some(p) => p,
        None => return Err(Error::new(ErrorKind::Other, format!("Can't determine points for game TarokGame::P")))
    };
    Ok(())
}

fn playB(
    players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    round: &i32, 
    radlci: HashMap<String, i32>,
    global_player_attributes: &mut HashMap<String, Vec<Option<TarokPlayerInput>>>, 
    global_game_attributes: &mut HashMap<String, Vec<Option<TarokGameInput>>>, 
    round_player_attributes: HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: Vec<TarokGameInput>
) -> Result<(), Error> {
    let points = match game_worth(TarokGame::B) {
        Some(p) => p,
        None => return Err(Error::new(ErrorKind::Other, format!("Can't determine points for game TarokGame::B")))
    };
    Ok(())
}

fn playKL(
    players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    round: &i32, 
    radlci: HashMap<String, i32>,
    global_player_attributes: &mut HashMap<String, Vec<Option<TarokPlayerInput>>>, 
    global_game_attributes: &mut HashMap<String, Vec<Option<TarokGameInput>>>, 
    round_player_attributes: HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: Vec<TarokGameInput>
) -> Result<(), Error> {
    let points = match game_worth(TarokGame::KL) {
        Some(p) => p,
        None => return Err(Error::new(ErrorKind::Other, format!("Can't determine points for game TarokGame::KL")))
    };
    Ok(())
}

fn playSB(
    players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    round: &i32, 
    radlci: HashMap<String, i32>,
    global_player_attributes: &mut HashMap<String, Vec<Option<TarokPlayerInput>>>, 
    global_game_attributes: &mut HashMap<String, Vec<Option<TarokGameInput>>>, 
    round_player_attributes: HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: Vec<TarokGameInput>
) -> Result<(), Error> {
    let points = match game_worth(TarokGame::SB) {
        Some(p) => p,
        None => return Err(Error::new(ErrorKind::Other, format!("Can't determine points for game TarokGame::SB")))
    };
    Ok(())
}

fn playS1(
    players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    round: &i32, 
    radlci: HashMap<String, i32>,
    global_player_attributes: &mut HashMap<String, Vec<Option<TarokPlayerInput>>>, 
    global_game_attributes: &mut HashMap<String, Vec<Option<TarokGameInput>>>, 
    round_player_attributes: HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: Vec<TarokGameInput>
) -> Result<(), Error> {
    let points = match game_worth(TarokGame::S1) {
        Some(p) => p,
        None => return Err(Error::new(ErrorKind::Other, format!("Can't determine points for game TarokGame::S1")))
    };
    Ok(())
}

fn playS2(
    players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    round: &i32, 
    radlci: HashMap<String, i32>,
    global_player_attributes: &mut HashMap<String, Vec<Option<TarokPlayerInput>>>, 
    global_game_attributes: &mut HashMap<String, Vec<Option<TarokGameInput>>>, 
    round_player_attributes: HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: Vec<TarokGameInput>
) -> Result<(), Error> {
    let points = match game_worth(TarokGame::S2) {
        Some(p) => p,
        None => return Err(Error::new(ErrorKind::Other, format!("Can't determine points for game TarokGame::S2")))
    };
    Ok(())
}

fn playS3(
    players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    round: &i32, 
    radlci: HashMap<String, i32>,
    global_player_attributes: &mut HashMap<String, Vec<Option<TarokPlayerInput>>>, 
    global_game_attributes: &mut HashMap<String, Vec<Option<TarokGameInput>>>, 
    round_player_attributes: HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: Vec<TarokGameInput>
) -> Result<(), Error> {
    let points = match game_worth(TarokGame::S3) {
        Some(p) => p,
        None => return Err(Error::new(ErrorKind::Other, format!("Can't determine points for game TarokGame::S3")))
    };
    Ok(())
}

fn playI1(
    players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    round: &i32, 
    radlci: HashMap<String, i32>,
    global_player_attributes: &mut HashMap<String, Vec<Option<TarokPlayerInput>>>, 
    global_game_attributes: &mut HashMap<String, Vec<Option<TarokGameInput>>>, 
    round_player_attributes: HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: Vec<TarokGameInput>
) -> Result<(), Error> {
    let points = match game_worth(TarokGame::I1) {
        Some(p) => p,
        None => return Err(Error::new(ErrorKind::Other, format!("Can't determine points for game TarokGame::I1")))
    };
    Ok(())
}

fn playI2(
    players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    round: &i32, 
    radlci: HashMap<String, i32>,
    global_player_attributes: &mut HashMap<String, Vec<Option<TarokPlayerInput>>>, 
    global_game_attributes: &mut HashMap<String, Vec<Option<TarokGameInput>>>, 
    round_player_attributes: HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: Vec<TarokGameInput>
) -> Result<(), Error> {
    let points = match game_worth(TarokGame::I2) {
        Some(p) => p,
        None => return Err(Error::new(ErrorKind::Other, format!("Can't determine points for game TarokGame::I2")))
    };
    Ok(())
}


fn playI3(
    players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    round: &i32, 
    radlci: HashMap<String, i32>,
    global_player_attributes: &mut HashMap<String, Vec<Option<TarokPlayerInput>>>, 
    global_game_attributes: &mut HashMap<String, Vec<Option<TarokGameInput>>>, 
    round_player_attributes: HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: Vec<TarokGameInput>
) -> Result<(), Error> {
    let points = match game_worth(TarokGame::I3) {
        Some(p) => p,
        None => return Err(Error::new(ErrorKind::Other, format!("Can't determine points for game TarokGame::I2")))
    };
    Ok(())
}

fn find_tarok_game(round_game_attributes: &[TarokGameInput]) -> Option<TarokGame> {
    for input in round_game_attributes.iter() {
        if let TarokGameInput::TarokGame(game) = input {
            return Some(game.clone());
        }
    }
    None
}

fn game_worth(g: TarokGame) -> Option<i32> {
    match g {
        TarokGame::I3 => Some(10),
        TarokGame::I2 => Some(20),
        TarokGame::I1 => Some(30),
        TarokGame::S3 => Some(40),
        TarokGame::S2 => Some(50),
        TarokGame::S1 => Some(60),
        TarokGame::SB => Some(80),
        TarokGame::KL => Some(0),
        TarokGame::B => Some(70),
        TarokGame::P => Some(60),
        TarokGame::BVI3 => Some(90),
        TarokGame::BVI2 => Some(100),
        TarokGame::BVI1 => Some(110),
        TarokGame::BVS3 => Some(120),
        TarokGame::BVS2 => Some(130),
        TarokGame::BVS1 => Some(140),
        TarokGame::BVSB => Some(150),
        _ => None
    }
}