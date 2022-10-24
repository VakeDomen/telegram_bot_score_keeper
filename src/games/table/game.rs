use std::{collections::HashMap, io::{ErrorKind, Error}};

use chrono::Utc;

use crate::{core::{traits::{CheckName, Game}, message_helper::extract_message_text, database::user_operations::get_user_by_name}, models::user::User};

use super::html_helper::build_score_table_html;

pub struct Table {
    players: Vec<User>,
    score: HashMap<String, Vec<Option<i32>>>,
    round: i32,
}

impl Table {
    pub fn new() -> Self {
        Self {
            players: vec![],
            score: HashMap::new(),
            round: 0 
        }
    }
}

impl CheckName for Table {}
impl Game for Table {
    
    fn start_game(&mut self) -> Result<String, std::io::Error> {
        Ok("Started a generic game score table!".to_string())
    }

    fn handle_round(&mut self, message: teloxide::types::Message) -> Result<String, Error> {
        let text = match extract_message_text(&message) {
            Some(text) => text,
            None => return Err(Error::new(ErrorKind::Other, "Failed to extract message text".to_string()))
        };
        let users = match extract_round_users(text.clone()) {
            Ok(users) => users,
            Err(e) => return Err(e),
        };

        let scores = match extract_round_scores(text) {
            Ok(score) => score,
            Err(e) => return Err(e),
        };
        if scores.len() != users.len() {
            return Err(Error::new(ErrorKind::Other, "Number of users and scores do not match".to_string()));
        }
        for index in 0..users.len() {
            let uid = users[index].id.clone();
            
            // if user seen for the first time fill the score with null until this round
            if !self.players.contains(&users[index]) {
                self.players.push(users[index].clone());
                let mut score: Vec<Option<i32>> = vec![];
                if self.round > 0 {
                    for _ in 0..self.round { score.push(None); }
                }
                self.score.insert(uid.clone(), score);
            }
            
            //insert score for this round
            if let Some(score) = self.score.get_mut(&uid) {
                // check for gaps in score (fill gaps with None)
                if score.len() < (self.round) as usize {
                    for _ in score.len()..(self.round) as usize {
                        score.push(None);
                    }
                }
                // push latest
                score.push(Some(scores[index]))
            } else {
                return Err(Error::new(ErrorKind::Other, format!("Something went wrong on entering user {} score for the round", uid)))
            }

        }
        self.round += 1;
        Ok(format!("{:#?}", self.score))
    }

    fn end_game(mut self: Box<Self>) -> Result<String, std::io::Error> {
        for player in self.players.iter() {
            if let Some(score) = self.score.get_mut(&player.id.to_string()) {
                if score.len() < (self.round) as usize {
                    for _ in score.len()..(self.round) as usize {
                        score.push(None);
                    }
                }
            } else {
                return Err(Error::new(ErrorKind::Other, format!("Something went wrong on entering user {} score for the missing rounds", player.name)))
            };
        }
        Ok(build_score_table_html(self.players, self.score, self.round))
    }

    fn get_state(&mut self) -> Result<String, std::io::Error> {
        todo!()
    }

    fn generate_file_name(&self) -> String {
        format!("{}_table.html", Utc::now())
    }
}

fn extract_round_users(message_text: String) -> Result<Vec<User>, Error> {
    let mut users = vec![];
    for fragment in message_text
        .split(' ')
        .skip(1)
        .step_by(2) 
    {
        let user_option = match get_user_by_name(fragment.to_uppercase()) {
            Ok(data) => data,
            Err(e) => return Err(Error::new(ErrorKind::Other, format!("Error fetching user from DB: {}", e))),
        };
        let user = match user_option {
            Some(user) => user,
            None => return Err(Error::new(ErrorKind::Other, "Error fetching user from DB".to_string())),
        };
        users.push(user);
    }
    Ok(users)
}

fn extract_round_scores(message_text: String) -> Result<Vec<i32>, Error> {
    let mut scores: Vec<i32> = vec![];
    for fragment in message_text
        .split(' ')
        .skip(2)
        .step_by(2)
    {
        let score = match fragment.parse() {
            Ok(num) => num,
            Err(_e) => return Err(Error::new(ErrorKind::Other, format!("Error parsing score {}", fragment))),
        };
        scores.push(score);
    }
    Ok(scores)
}