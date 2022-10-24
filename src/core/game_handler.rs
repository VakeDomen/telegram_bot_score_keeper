use std::collections::HashMap;

use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use super::traits::Game;

pub static RUNNING_GAMES: Lazy<Mutex<HashMap<String, Box<dyn Game + Send >>>> = Lazy::new(|| {Mutex::new(HashMap::new())});