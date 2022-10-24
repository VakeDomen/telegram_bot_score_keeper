use std::collections::HashMap;

use crate::models::user::User;

pub fn build_score_table_html(players: Vec<User>, score_table: HashMap<String, Vec<Option<i32>>>, rounds: i32) -> String {
    format!(
        "{}{}{}", 
        get_html_head(), 
        generate_table(&score_table, &players, rounds), 
        get_html_tail()
    )
}

fn generate_table(score_table: &HashMap<String, Vec<Option<i32>>>, players: &[User], rounds: i32) -> String {
    let mut table = String::from("");
    // generate table header
    for player in players.iter() {
        let append = format!("<th>{}</th>", player.name);
        table = format!("{}{}", table,append)
    }
    table = format!("<tr>{}</tr>", table);
    // generate table rows 
    for index in 0..rounds {
        table = format!("{}<tr>{}</tr>", table, generate_line(score_table, players, index));
    }
    table
}

fn generate_line(score_table: &HashMap<String, Vec<Option<i32>>>, players: &[User], index: i32) -> String {
    let mut line = String::from("");
    for player in players.iter() {
        let append = format!("<td>{}</td>", extract_field_value(score_table, player, index));
        line = format!("{}{}", line, append);
    }
    line
}

fn extract_field_value(score_table: &HashMap<String, Vec<Option<i32>>>, player: &User, index: i32) -> String {
    match score_table.get(&player.id) {
        Some(score) => match score[index as usize] {
            Some(val) => format!("{}", val),
            None => "".to_string(),
        },
        None => "Missing".to_string(),
    }
}

fn get_html_tail() -> String {
    "</table></body></html>".to_string()
}

fn get_html_head() -> String {
    "<!DOCTYPE html><html lang='en'><head><meta charset='UTF-8'><meta http-equiv='X-UA-Compatible' content='IE=edge'>
    <meta name='viewport' content='width=device-width, initial-scale=1.0'><title>Document</title></head><body><style>
    table{width: 100%;text-align: center;}tr:nth-child(2n) {color: rgb(128, 128, 128)}th {color: #a6acf3;}.biggest {
    color: green;}.smallest {color: red}td,th {border: 1px solid rgb(190, 190, 190);}</style><table>".to_string()
}