use std::io::Error;

use teloxide::types::Message;

pub trait CheckName {
    fn is_valid_name(&self, name: &str) -> bool { !self.get_reserved_terms().contains(&name) }
    fn get_reserved_terms(&self) -> &'static [&'static str] { &[] }
}

pub trait Game {
    fn start_game(&mut self) -> Result<String, Error>;
    fn handle_round(&mut self, message: Message) -> Result<String, Error>;
    fn end_game(self: Box<Self>) -> Result<String, Error>; // https://stackoverflow.com/questions/63766721/size-of-dyn-mytrait-cannot-be-statically-determined-in-method-which-takes-self
    fn get_state(&mut self) -> Result<String, Error>;
}