use crate::core::traits::{CheckName, Game};

pub struct Tarok;

impl CheckName for Tarok {
    fn get_reserved_terms(&self) -> &'static [&'static str] {
        &["3","2", "1", "S3", "S2", "S1"]
    }
}


impl Game for Tarok {
    fn start_game(&mut self) -> Result<String, std::io::Error> {
        todo!()
    }

    fn handle_round(&mut self, message: teloxide::types::Message) -> Result<String, std::io::Error> {
        todo!()
    }

    fn end_game(&mut self) -> Result<String, std::io::Error> {
        todo!()
    }

    fn get_state(&mut self) -> Result<String, std::io::Error> {
        todo!()
    }
}