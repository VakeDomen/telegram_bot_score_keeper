use std::io::Error;

use teloxide::types::Message;

pub trait CheckName {
    fn is_valid_name(&self, name: &str) -> bool { !self.get_reserved_terms().contains(&name) }
    fn get_reserved_terms(&self) -> &'static [&'static str];
}

pub trait Game {
    fn start_game(&mut self) -> Result<String, Error>;
    fn handle_round(&mut self, message: Message) -> Result<String, Error>;
    fn end_game(&mut self) -> Result<String, Error>;
    fn get_state(&mut self) -> Result<String, Error>;
}