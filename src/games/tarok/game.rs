use std::{collections::HashMap, io::{Error, ErrorKind}};
use chrono::Utc;

use crate::{core::{traits::{CheckName, Game}, message_helper::extract_message_text, database::user_operations::get_user_by_name}, models::{user::User}};

use super::enums::{TarokGameInput, TarokGame, TarokGameAttribute, TarokPlayerAttibute, TarokPlayerInput, Radlc};

pub struct Tarok {
    players: Vec<User>,
    radlci: HashMap<String, Vec<Radlc>>,
    score: HashMap<String, Vec<Option<i32>>>,
    player_attributes: HashMap<String, Vec<Option<Vec<TarokPlayerInput>>>>,
    game_attributes: Vec<Vec<TarokGameInput>>,
    round: i32,
}

impl Tarok {
    pub fn new() -> Self {
        Self {
            players: vec![],
            radlci: HashMap::new(),
            score: HashMap::new(),
            player_attributes: HashMap::new(),
            game_attributes: Vec::new(),
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
            &mut self.player_attributes,
            &self.round,
            &mut self.radlci,
        );
        
        let mut game_attributes: Vec<TarokGameInput> = match extract_game_attributes(&text) {
            Ok(attr) => attr,
            Err(e) => return Err(Error::new(ErrorKind::Other, format!("Failed to extract game attributes: {}", e)))
        };

        let mut player_attributes: HashMap<String, Vec<TarokPlayerInput>> = match extract_player_attributes(&self.players, &text) {
            Ok(attr) => attr,
            Err(e) => return Err(Error::new(ErrorKind::Other, format!("Failed to extract player attributes: {}", e)))
        };

        let status = match handle_game(
            &users,
            &mut self.score, 
            &mut self.radlci,
            &mut player_attributes,
            &mut game_attributes,
        ) {
            Ok(st) => st,
            Err(e) => return Err(Error::new(ErrorKind::Other, format!("Failed to calculate round: {}", e)))
        };
        increment_round(&mut self.round);
        // save game attributes and player attributes to global sheets
        if let Err(e) = save_round_to_sheets(
            player_attributes, 
            &mut self.player_attributes, 
            game_attributes, 
            &mut self.game_attributes
        ) {
        return Err(Error::new(ErrorKind::Other, format!("Error saving round: {}", e)))
    };
        Ok(format!("{:#?} \n{:#?}", status, self.radlci))
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
        "ZP" => Some(TarokGameAttribute::ZP),
        "ZK" => Some(TarokGameAttribute::ZK),
        "V" => Some(TarokGameAttribute::V),
        "T" => Some(TarokGameAttribute::T),
        "K" => Some(TarokGameAttribute::K),
        "NZP" => Some(TarokGameAttribute::NZP),
        "NZK" => Some(TarokGameAttribute::NZK),
        "NV" => Some(TarokGameAttribute::NV),
        "NT" => Some(TarokGameAttribute::NT),
        "NK" => Some(TarokGameAttribute::NK),
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
    global_player_attributes: &mut HashMap<String, Vec<Option<Vec<TarokPlayerInput>>>>,
    round: &i32,
    radlci: &mut HashMap<String, Vec<Radlc>>,
) {
    for user in users.iter() {
        if players.contains(user) {
            continue;
        }
        players.push(user.clone());
        let mut player_score = vec![];
        fill_gaps_until_round(&mut player_score, round);
        score.insert(user.id.clone(), player_score);
        global_player_attributes.insert(user.id.clone(), vec![]);
        radlci.insert(user.id.clone(), vec![]);
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
    round_players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    radlci: &mut HashMap<String, Vec<Radlc>>,
    round_player_attributes: &mut HashMap<String, Vec<TarokPlayerInput>>,
    round_game_attributes: &mut Vec<TarokGameInput>,
) -> Result<HashMap<String, i32>, Error> {
    // find what game we are playing
    let game: TarokGame = match find_tarok_game(&round_game_attributes) {
        Some(game) => game,
        None => return Err(Error::new(ErrorKind::Other, "No game specified".to_string())),
    };
    match game {
        TarokGame::I3 => playI3(round_players, score, radlci, round_player_attributes, round_game_attributes),
        TarokGame::I2 => playI2(round_players, score, radlci, round_player_attributes, round_game_attributes),
        TarokGame::I1 => playI1(round_players, score, radlci, round_player_attributes, round_game_attributes),
        TarokGame::S3 => playS3(round_players, score, radlci, round_player_attributes, round_game_attributes),
        TarokGame::S2 => playS2(round_players, score, radlci, round_player_attributes, round_game_attributes),
        TarokGame::S1 => playS1(round_players, score, radlci, round_player_attributes, round_game_attributes),
        TarokGame::SB => playSB(round_players, score, radlci, round_player_attributes, round_game_attributes),
        TarokGame::KL => playKL(round_players, score, radlci, round_player_attributes, round_game_attributes),
        TarokGame::B => playB(round_players, score, radlci, round_player_attributes, round_game_attributes),
        TarokGame::P => playP(round_players, score, radlci, round_player_attributes, round_game_attributes),
        TarokGame::BVI3 => playBVI3(round_players, score, radlci, round_player_attributes, round_game_attributes),
        TarokGame::BVI2 => playBVI2(round_players, score, radlci, round_player_attributes, round_game_attributes),
        TarokGame::BVI1 => playBVI1(round_players, score, radlci, round_player_attributes, round_game_attributes),
        TarokGame::BVS3 => playBVS3(round_players, score, radlci, round_player_attributes, round_game_attributes),
        TarokGame::BVS2 => playBVS2(round_players, score, radlci, round_player_attributes, round_game_attributes),
        TarokGame::BVS1 => playBVS1(round_players, score, radlci, round_player_attributes, round_game_attributes),
        TarokGame::BVSB => playBVSB(round_players, score, radlci, round_player_attributes, round_game_attributes),
    }
}

fn playBVSB(
    round_players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    radlci: &mut HashMap<String, Vec<Radlc>>,
    round_player_attributes: &mut HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: &mut Vec<TarokGameInput>
) -> Result<HashMap<String, i32>, Error> {
    // check if at least one player exists
    if let Err(e) = players_validity_check(round_players) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }
    // get points of the game
    let mut game_points = calculate_base_game_points(&round_game_attributes);
    
    // add the attribute of "playing player" to the first player
    if let Err(e) = add_playing_attribute_to_first_player(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }

    // check if player that is playing the round (should be first) has a radlc avalible
    // if yes double game points
    handle_radlc(round_players, radlci, &mut game_points);
    
    let changes = match score_game_and_player(round_players, score, &round_player_attributes, &game_points) {
        Ok(hm) => hm,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Error caluclating score: {}", e))),
    };
    // add radlc to all players
    add_radlc(radlci);
    Ok(changes)
}

fn playBVS1(
    round_players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    radlci: &mut HashMap<String, Vec<Radlc>>,
    round_player_attributes: &mut HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: &mut Vec<TarokGameInput>
) -> Result<HashMap<String, i32>, Error> {
    // check if at least one player exists
    if let Err(e) = players_validity_check(round_players) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }
    // get points of the game
    let mut game_points = calculate_base_game_points(&round_game_attributes);
    
    // add the attribute of "playing player" to the first player
    if let Err(e) = add_playing_attribute_to_first_player(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }

    // check if player that is playing the round (should be first) has a radlc avalible
    // if yes double game points
    handle_radlc(round_players, radlci, &mut game_points);
    
    let changes = match score_game_and_player(round_players, score, &round_player_attributes, &game_points) {
        Ok(hm) => hm,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Error caluclating score: {}", e))),
    };
    // add radlc to all players
    add_radlc(radlci);
    Ok(changes)
}

fn playBVS2(
    round_players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    radlci: &mut HashMap<String, Vec<Radlc>>,
    round_player_attributes: &mut HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: &mut Vec<TarokGameInput>
) -> Result<HashMap<String, i32>, Error> {
    // check if at least one player exists
    if let Err(e) = players_validity_check(round_players) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }
    // get points of the game
    let mut game_points = calculate_base_game_points(&round_game_attributes);
    
    // add the attribute of "playing player" to the first player
    if let Err(e) = add_playing_attribute_to_first_player(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }

    // check if player that is playing the round (should be first) has a radlc avalible
    // if yes double game points
    handle_radlc(round_players, radlci, &mut game_points);
    
    let changes = match score_game_and_player(round_players, score, &round_player_attributes, &game_points) {
        Ok(hm) => hm,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Error caluclating score: {}", e))),
    };
    // add radlc to all players
    add_radlc(radlci);
    Ok(changes)
}

fn playBVS3(
    round_players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    radlci: &mut HashMap<String, Vec<Radlc>>,
    round_player_attributes: &mut HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: &mut Vec<TarokGameInput>
) -> Result<HashMap<String, i32>, Error> {
    // check if at least one player exists
    if let Err(e) = players_validity_check(round_players) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }
    // get points of the game
    let mut game_points = calculate_base_game_points(&round_game_attributes);
    
    // add the attribute of "playing player" to the first player
    if let Err(e) = add_playing_attribute_to_first_player(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }

    // check if player that is playing the round (should be first) has a radlc avalible
    // if yes double game points
    handle_radlc(round_players, radlci, &mut game_points);
    
    let changes = match score_game_and_player(round_players, score, &round_player_attributes, &game_points) {
        Ok(hm) => hm,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Error caluclating score: {}", e))),
    };
    // add radlc to all players
    add_radlc(radlci);
    Ok(changes)
}

fn playBVI1(
    round_players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    radlci: &mut HashMap<String, Vec<Radlc>>,
    round_player_attributes: &mut HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: &mut Vec<TarokGameInput>
) -> Result<HashMap<String, i32>, Error> {
    // check if at least one player exists
    if let Err(e) = players_validity_check(round_players) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }
    // get points of the game
    let mut game_points = calculate_base_game_points(&round_game_attributes);
    
    // add the attribute of "playing player" to the first player
    if let Err(e) = add_playing_attribute_to_first_player(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }

    // add attribute of "supporting player" to other players
    if let Err(e) = add_supporting_attribute_to_players(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }

    // check if player that is playing the round (should be first) has a radlc avalible
    // if yes double game points
    handle_radlc(round_players, radlci, &mut game_points);
    
    let changes = match score_game_and_player(round_players, score, &round_player_attributes, &game_points) {
        Ok(hm) => hm,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Error caluclating score: {}", e))),
    };
    // add radlc to all players
    add_radlc(radlci);
    Ok(changes)
}

fn playBVI2(
    round_players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    radlci: &mut HashMap<String, Vec<Radlc>>,
    round_player_attributes: &mut HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: &mut Vec<TarokGameInput>
) -> Result<HashMap<String, i32>, Error> {
    // check if at least one player exists
    if let Err(e) = players_validity_check(round_players) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }
    // get points of the game
    let mut game_points = calculate_base_game_points(&round_game_attributes);
    
    // add the attribute of "playing player" to the first player
    if let Err(e) = add_playing_attribute_to_first_player(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }

    // add attribute of "supporting player" to other players
    if let Err(e) = add_supporting_attribute_to_players(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }

    // check if player that is playing the round (should be first) has a radlc avalible
    // if yes double game points
    handle_radlc(round_players, radlci, &mut game_points);
    
    let changes = match score_game_and_player(round_players, score, &round_player_attributes, &game_points) {
        Ok(hm) => hm,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Error caluclating score: {}", e))),
    };
    // add radlc to all players
    add_radlc(radlci);
    Ok(changes)
}

fn playBVI3(
    round_players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    radlci: &mut HashMap<String, Vec<Radlc>>,
    round_player_attributes: &mut HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: &mut Vec<TarokGameInput>
) -> Result<HashMap<String, i32>, Error> {
    // check if at least one player exists
    if let Err(e) = players_validity_check(round_players) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }
    // get points of the game
    let mut game_points = calculate_base_game_points(&round_game_attributes);
    
    // add the attribute of "playing player" to the first player
    if let Err(e) = add_playing_attribute_to_first_player(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }

    // add attribute of "supporting player" to other players
    if let Err(e) = add_supporting_attribute_to_players(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }

    // check if player that is playing the round (should be first) has a radlc avalible
    // if yes double game points
    handle_radlc(round_players, radlci, &mut game_points);
    
    let changes = match score_game_and_player(round_players, score, &round_player_attributes, &game_points) {
        Ok(hm) => hm,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Error caluclating score: {}", e))),
    };
    // add radlc to all players
    add_radlc(radlci);
    Ok(changes)
}

fn playP(
    round_players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    radlci: &mut HashMap<String, Vec<Radlc>>,
    round_player_attributes: &mut HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: &mut Vec<TarokGameInput>
) -> Result<HashMap<String, i32>, Error> {
    // check if at least one player exists
    if let Err(e) = players_validity_check(round_players) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }
    // get points of the game
    let mut game_points = calculate_base_game_points(&round_game_attributes);
    
    // add the attribute of "playing player" to the first player
    if let Err(e) = add_playing_attribute_to_first_player(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }

    // check if player that is playing the round (should be first) has a radlc avalible
    // if yes double game points
    handle_radlc(round_players, radlci, &mut game_points);
    
    let changes = match score_game_and_player(round_players, score, &round_player_attributes, &game_points) {
        Ok(hm) => hm,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Error caluclating score: {}", e))),
    };
    // add radlc to all players
    add_radlc(radlci);
    Ok(changes)
}

fn playB(
    round_players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    radlci: &mut HashMap<String, Vec<Radlc>>,
    round_player_attributes: &mut HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: &mut Vec<TarokGameInput>
) -> Result<HashMap<String, i32>, Error> {
    // check if at least one player exists
    if let Err(e) = players_validity_check(round_players) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }
    // get points of the game
    let mut game_points = calculate_base_game_points(&round_game_attributes);
    
    // add the attribute of "playing player" to the first player
    if let Err(e) = add_playing_attribute_to_first_player(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }

    // check if player that is playing the round (should be first) has a radlc avalible
    // if yes double game points
    handle_radlc(round_players, radlci, &mut game_points);
    
    let changes = match score_game_and_player(round_players, score, &round_player_attributes, &game_points) {
        Ok(hm) => hm,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Error caluclating score: {}", e))),
    };
    // add radlc to all players
    add_radlc(radlci);
    Ok(changes)
}

fn playKL(
    round_players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    radlci: &mut HashMap<String, Vec<Radlc>>,
    round_player_attributes: &mut HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: &mut Vec<TarokGameInput>
) -> Result<HashMap<String, i32>, Error> {
    // check if at least one player exists
    if let Err(e) = players_validity_check(round_players) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }
    // add the attribute of "playing player" to the first player
    if let Err(e) = add_playing_attribute_to_first_player(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }

    let changes = match score_player_only(round_players, score, &round_player_attributes) {
        Ok(hm) => hm,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Error caluclating score: {}", e))),
    };
    // add radlc to all players
    add_radlc(radlci);
    Ok(changes)
}

fn playSB(
    round_players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    radlci: &mut HashMap<String, Vec<Radlc>>,
    round_player_attributes: &mut HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: &mut Vec<TarokGameInput>
) -> Result<HashMap<String, i32>, Error> {
    // check if at least one player exists
    if let Err(e) = players_validity_check(round_players) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }
    // get points of the game
    let mut game_points = calculate_base_game_points(&round_game_attributes);
    
    // add the attribute of "playing player" to the first player
    if let Err(e) = add_playing_attribute_to_first_player(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }

    // check if player that is playing the round (should be first) has a radlc avalible
    // if yes double game points
    handle_radlc(round_players, radlci, &mut game_points);
    
    let changes = match score_game_and_player(round_players, score, &round_player_attributes, &game_points) {
        Ok(hm) => hm,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Error caluclating score: {}", e))),
    };
    // add radlc to all players
    add_radlc(radlci);
    Ok(changes)
}

fn playS1(
    round_players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    radlci: &mut HashMap<String, Vec<Radlc>>,
    round_player_attributes: &mut HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: &mut Vec<TarokGameInput>
) -> Result<HashMap<String, i32>, Error> {
    // check if at least one player exists
    if let Err(e) = players_validity_check(round_players) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }
    // get points of the game
    let mut game_points = calculate_base_game_points(&round_game_attributes);
    
    // add the attribute of "playing player" to the first player
    if let Err(e) = add_playing_attribute_to_first_player(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }
    // check if player that is playing the round (should be first) has a radlc avalible
    // if yes double game points
    handle_radlc(round_players, radlci, &mut game_points);
    
    let changes = match score_game_and_player(round_players, score, &round_player_attributes, &game_points) {
        Ok(hm) => hm,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Error caluclating score: {}", e))),
    };
    Ok(changes)
}

fn playS2(
    round_players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    radlci: &mut HashMap<String, Vec<Radlc>>,
    round_player_attributes: &mut HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: &mut Vec<TarokGameInput>
) -> Result<HashMap<String, i32>, Error> {
    // check if at least one player exists
    if let Err(e) = players_validity_check(round_players) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }
    // get points of the game
    let mut game_points = calculate_base_game_points(&round_game_attributes);
    
    // add the attribute of "playing player" to the first player
    if let Err(e) = add_playing_attribute_to_first_player(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }
    // check if player that is playing the round (should be first) has a radlc avalible
    // if yes double game points
    handle_radlc(round_players, radlci, &mut game_points);
    
    let changes = match score_game_and_player(round_players, score, &round_player_attributes, &game_points) {
        Ok(hm) => hm,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Error caluclating score: {}", e))),
    };
    Ok(changes)
}

fn playS3(
    round_players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    radlci: &mut HashMap<String, Vec<Radlc>>,
    round_player_attributes: &mut HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: &mut Vec<TarokGameInput>
) -> Result<HashMap<String, i32>, Error> {
    // check if at least one player exists
    if let Err(e) = players_validity_check(round_players) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }
    // get points of the game
    let mut game_points = calculate_base_game_points(&round_game_attributes);
    
    // add the attribute of "playing player" to the first player
    if let Err(e) = add_playing_attribute_to_first_player(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }
    // check if player that is playing the round (should be first) has a radlc avalible
    // if yes double game points
    handle_radlc(round_players, radlci, &mut game_points);
    
    let changes = match score_game_and_player(round_players, score, &round_player_attributes, &game_points) {
        Ok(hm) => hm,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Error caluclating score: {}", e))),
    };
    Ok(changes)
}

fn playI1(
    round_players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    radlci: &mut HashMap<String, Vec<Radlc>>,
    round_player_attributes: &mut HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: &mut Vec<TarokGameInput>
) -> Result<HashMap<String, i32>, Error> {
    // check if at least one player exists
    if let Err(e) = players_validity_check(round_players) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }
    // get points of the game
    let mut game_points = calculate_base_game_points(&round_game_attributes);
    
    // add the attribute of "playing player" to the first player
    if let Err(e) = add_playing_attribute_to_first_player(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }

    // add attribute of "supporting player" to other players
    if let Err(e) = add_supporting_attribute_to_players(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }

    // check if player that is playing the round (should be first) has a radlc avalible
    // if yes double game points
    handle_radlc(round_players, radlci, &mut game_points);
    
    let changes = match score_game_and_player(round_players, score, &round_player_attributes, &game_points) {
        Ok(hm) => hm,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Error caluclating score: {}", e))),
    };
    Ok(changes)
}

fn playI2(
    round_players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    radlci: &mut HashMap<String, Vec<Radlc>>,
    round_player_attributes: &mut HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: &mut Vec<TarokGameInput>
) -> Result<HashMap<String, i32>, Error> {
    // check if at least one player exists
    if let Err(e) = players_validity_check(round_players) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }
    // get points of the game
    let mut game_points = calculate_base_game_points(&round_game_attributes);
    
    // add the attribute of "playing player" to the first player
    if let Err(e) = add_playing_attribute_to_first_player(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }

    // add attribute of "supporting player" to other players
    if let Err(e) = add_supporting_attribute_to_players(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }

    // check if player that is playing the round (should be first) has a radlc avalible
    // if yes double game points
    handle_radlc(round_players, radlci, &mut game_points);
    
    let changes = match score_game_and_player(round_players, score, &round_player_attributes, &game_points) {
        Ok(hm) => hm,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Error caluclating score: {}", e))),
    };
    Ok(changes)
}


fn playI3(
    round_players: &[User], 
    score: &mut HashMap<String, Vec<Option<i32>>>, 
    radlci: &mut HashMap<String, Vec<Radlc>>,
    round_player_attributes: &mut HashMap<String, Vec<TarokPlayerInput>>, 
    round_game_attributes: &mut Vec<TarokGameInput>
) -> Result<HashMap<String, i32>, Error> {
    // check if at least one player exists
    if let Err(e) = players_validity_check(round_players) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }
    // get points of the game
    let mut game_points = calculate_base_game_points(&round_game_attributes);
    
    // add the attribute of "playing player" to the first player
    if let Err(e) = add_playing_attribute_to_first_player(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }

    // add attribute of "supporting player" to other players
    if let Err(e) = add_supporting_attribute_to_players(round_players, round_player_attributes) {
        return Err(Error::new(ErrorKind::Other, format!("{}", e)));
    }

    // check if player that is playing the round (should be first) has a radlc avalible
    // if yes double game points
    handle_radlc(round_players, radlci, &mut game_points);
    
    let changes = match score_game_and_player(round_players, score, &round_player_attributes, &game_points) {
        Ok(hm) => hm,
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Error caluclating score: {}", e))),
    };
    Ok(changes)
}

fn score_game_and_player(
    players: &[User],
    score: &mut HashMap<String, Vec<Option<i32>>>,
    round_player_attributes: &HashMap<String, Vec<TarokPlayerInput>>,
    game_points: &i32
) -> Result<HashMap<String, i32>, Error>{
    let mut score_change = HashMap::new();
    for player in players.iter() {
        // get player attributes
        let attrs = match round_player_attributes.get(&player.id) {
            Some(att) => att,
            None => return Err(Error::new(ErrorKind::Other, format!("Player does not have attribute vector!"))),
        };
        // calc player personal score modifiers (lost mond, support,...)
        let mut personal_points = 0;
        for p_attr in attrs.iter() {
            personal_points += player_attribute_worth(p_attr)
        }

        // save player score to the game score sheet
        match score.get_mut(&player.id) {
            Some(sc) => {
                score_change.insert(player.name.clone(), game_points + personal_points);
                sc.push(Some(game_points + personal_points))
            },
            None => return Err(Error::new(ErrorKind::Other, format!("Player does not have a score vector!"))),
        };
    }
    Ok(score_change)
}

fn score_game_only(
    players: &[User],
    score: &mut HashMap<String, Vec<Option<i32>>>,
    game_points: &i32
) -> Result<HashMap<String, i32>, Error>{
    let mut score_change = HashMap::new();
    for player in players.iter() {
        // save player score to the game score sheet
        match score.get_mut(&player.id) {
            Some(sc) => {
                score_change.insert(player.name.clone(), *game_points);
                sc.push(Some(*game_points))
            },
            None => return Err(Error::new(ErrorKind::Other, format!("Player does not have a score vector!"))),
        };
    }
    Ok(score_change)
}

fn score_player_only(
    players: &[User],
    score: &mut HashMap<String, Vec<Option<i32>>>,
    round_player_attributes: &HashMap<String, Vec<TarokPlayerInput>>,
) -> Result<HashMap<String, i32>, Error>{
    let mut score_change = HashMap::new();
    for player in players.iter() {
        // get player attributes
        let attrs = match round_player_attributes.get(&player.id) {
            Some(att) => att,
            None => return Err(Error::new(ErrorKind::Other, format!("Player does not have attribute vector!"))),
        };
        // calc player personal score modifiers (lost mond, support,...)
        let mut personal_points = 0;
        for p_attr in attrs.iter() {
            personal_points += player_attribute_worth(p_attr)
        }

        // save player score to the game score sheet
        match score.get_mut(&player.id) {
            Some(sc) => {
                score_change.insert(player.name.clone(), personal_points);
                sc.push(Some(personal_points))
            },
            None => return Err(Error::new(ErrorKind::Other, format!("Player does not have a score vector!"))),
        };
    }
    Ok(score_change)
}


fn increment_round(round: &mut i32) {
    *round += 1;
}

fn handle_radlc(
    players: &[User],
    radlci: &mut HashMap<String, Vec<Radlc>>,
    game_points: &mut i32
) {
    if let true = player_has_avalible_radlc(&players[0].id, radlci) {
        *game_points *= 2;
        consume_player_radlc(&players[0].id, radlci);
    }
}

fn add_radlc(
    radlci: &mut HashMap<String, Vec<Radlc>>
) {
    for (_, rad) in radlci.iter_mut() {
        rad.push(Radlc::Avalible);
    }
}

fn add_supporting_attribute_to_players(
    round_players: &[User],
    round_player_attributes: &mut HashMap<String, Vec<TarokPlayerInput>>,
) -> Result<(), Error> {
    // add attribute of "supporting player" to other players
    for player in round_players.iter().skip(1) {
        let attr = match round_player_attributes.get_mut(&player.id) {
            Some(att) => att,
            None => return Err(Error::new(ErrorKind::Other, format!("Player does not have attribute vector!"))),
        };
        attr.push(TarokPlayerInput::PlayerAttribute(TarokPlayerAttibute::Sl));
    }
    Ok(())
}

fn add_playing_attribute_to_first_player(
    players: &[User], 
    round_player_attributes: &mut HashMap<String, Vec<TarokPlayerInput>>
) -> Result<(), Error> {
    // add the attribute of "playing player" to the first player
    match round_player_attributes.get_mut(&players[0].id) {
        Some(att) => Ok(att.push(TarokPlayerInput::PlayerAttribute(TarokPlayerAttibute::Ig))),
        None => return Err(Error::new(ErrorKind::Other, format!("Player does not have attribute vector!"))),
    }
}

fn calculate_base_game_points(round_game_attributes: &Vec<TarokGameInput>) -> i32 {
    // get points of the game
    let mut game_points = 0;
    for g_attr in round_game_attributes.iter() {
        game_points += attribute_worth(g_attr)
    }
    game_points
}

fn players_validity_check(players: &[User]) -> Result<(), Error> {
    // check if at least one player exists
    if players.is_empty() {
        return Err(Error::new(ErrorKind::Other, format!("No players specified!")));
    }
    Ok(())
}

fn save_round_to_sheets(
    round_player_attributes: HashMap<String, Vec<TarokPlayerInput>>, 
    global_player_attributes: &mut HashMap<String, Vec<Option<Vec<TarokPlayerInput>>>>,
    round_game_attributes: Vec<TarokGameInput>, 
    global_game_attributes: &mut Vec<Vec<TarokGameInput>>
) -> Result<(), Error> {
    // save game attributes to global sheet
    match save_game_attributes(round_game_attributes, global_game_attributes) {
        Ok(_) => (),
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Failed saving game attributes to sheet: {}", e))),
    }

    // save player attributes to global sheet
    match save_player_attributes(round_player_attributes, global_player_attributes) {
        Ok(_) => (),
        Err(e) => return Err(Error::new(ErrorKind::Other, format!("Failed saving player attributes to sheet: {}", e))),
    }

    Ok(())
}

fn save_player_attributes(
    round_player_attributes: HashMap<String, Vec<TarokPlayerInput>>, 
    global_player_attributes: &mut HashMap<String, Vec<Option<Vec<TarokPlayerInput>>>>
) -> Result<(), Error> {
    for (player_id, attributes) in round_player_attributes.into_iter() {
        match global_player_attributes.get_mut(&player_id) {
            Some(sh) => sh.push(Some(attributes)),
            None => return Err(Error::new(ErrorKind::Other, format!("Player attribute sheet missing"))),
        }
    }
    Ok(())
}

fn save_game_attributes(
    round_game_attributes: Vec<TarokGameInput>, 
    global_game_attributes: &mut Vec<Vec<TarokGameInput>>
) -> Result<(), Error> {
    global_game_attributes.push(round_game_attributes);
    Ok(())
}

fn player_attribute_worth(attr: &TarokPlayerInput) -> i32 {
    match attr {
        TarokPlayerInput::PlayerAttribute(at) => match at {
            TarokPlayerAttibute::M => -20,
            TarokPlayerAttibute::R => 0,
            TarokPlayerAttibute::T => 0,
            TarokPlayerAttibute::Ig => 0,
            TarokPlayerAttibute::Sl => -20,
        },
        TarokPlayerInput::PlayerDiff(val) => *val,
    }
}

fn attribute_worth(g_attr: &TarokGameInput) -> i32 {
    match g_attr {
        TarokGameInput::TarokGame(g) => game_worth(*g),
        TarokGameInput::TarokGameDiff(val) => *val,
        TarokGameInput::TarokGameAttribute(att) => match att {
            TarokGameAttribute::ZP => 10,
            TarokGameAttribute::ZK => 10,
            TarokGameAttribute::V => 150,
            TarokGameAttribute::T => 15,
            TarokGameAttribute::K => 15,
            TarokGameAttribute::NZP => 20,
            TarokGameAttribute::NZK => 20,
            TarokGameAttribute::NV => 250,
            TarokGameAttribute::NT => 30,
            TarokGameAttribute::NK => 30,
        },
    }
}

fn find_tarok_game(round_game_attributes: &[TarokGameInput]) -> Option<TarokGame> {
    for input in round_game_attributes.iter() {
        if let TarokGameInput::TarokGame(game) = input {
            return Some(game.clone());
        }
    }
    None
}

fn game_worth(g: TarokGame) -> i32 {
    match g {
        TarokGame::I3 => 10,
        TarokGame::I2 => 20,
        TarokGame::I1 => 30,
        TarokGame::S3 => 40,
        TarokGame::S2 => 50,
        TarokGame::S1 => 60,
        TarokGame::SB => 80,
        TarokGame::KL => 0,
        TarokGame::B => 70,
        TarokGame::P => 60,
        TarokGame::BVI3 => 90,
        TarokGame::BVI2 => 100,
        TarokGame::BVI1 => 110,
        TarokGame::BVS3 => 120,
        TarokGame::BVS2 => 130,
        TarokGame::BVS1 => 140,
        TarokGame::BVSB => 150,
    }
}

fn player_has_avalible_radlc(player_id: &String, radlci: &mut HashMap<String, Vec<Radlc>>) -> bool {
    let player_radlci = match radlci.get(player_id) {
        Some(radlci) => radlci,
        None => return false,
    }; 
    if player_radlci.is_empty() {
        return false;
    }
    for radlc in player_radlci.iter() {
        if let Radlc::Avalible = radlc {
            return true
        }
    }
    false
}


fn consume_player_radlc(player_id: &String, radlci: &mut HashMap<String, Vec<Radlc>>) -> bool {
    let player_radlci = match radlci.get_mut(player_id) {
        Some(radlci) => radlci,
        None => return false,
    }; 
    if player_radlci.is_empty() {
        return false;
    }
    for radlc in player_radlci.iter_mut() {
        if let Radlc::Avalible = radlc {
            *radlc = Radlc::Used;
            return true
        }
    }
    false
}