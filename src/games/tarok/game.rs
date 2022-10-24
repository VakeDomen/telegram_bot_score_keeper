use std::collections::HashMap;

use crate::{core::traits::{CheckName, Game}, models::user::User};

pub struct Tarok {
    _players: Vec<User>,
    _radlci: HashMap<String, i32>,
    _score: HashMap<String, i32>,
}

impl Tarok {
    pub fn new() -> Self {
        Self {
            _players: vec![],
            _radlci: HashMap::new(),
            _score: HashMap::new(),
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

    fn handle_round(&mut self, _message: teloxide::types::Message) -> Result<String, std::io::Error> {
        todo!()
    }

    fn end_game(self: Box<Self>) -> Result<String, std::io::Error> {
        todo!()
    }

    fn get_state(&mut self) -> Result<String, std::io::Error> {
        todo!()
    }

    fn generate_file_name(&self) -> String {
        todo!()
    }
}