use std::collections::HashMap;

use crate::models::user::User;

use super::enums::{Radlc, TarokGameInput, TarokPlayerInput};

pub fn build_score_table_html(
    players: &[User], 
    score: &HashMap<String, Vec<Option<i32>>>, 
    rounds: i32, 
    sum_by_player: HashMap<String, (i32, i32, i32)>,
    radlci: &HashMap<String, Vec<Radlc>>,
    global_player_attributes: &mut HashMap<String, Vec<Option<Vec<TarokPlayerInput>>>>,
    global_game_attributes: &mut Vec<Vec<TarokGameInput>>,
) -> String {
    let mut table = String::from("");
    // generate table header
    for player in players.iter() {
        let append = format!("<th>{}</th>", player.name);
        table = format!("{}{}", table,append)
    }
    table = format!("<tr>{}</tr>", table);

    // generate radlc row
    let mut line = "".to_string();
    for player in players.iter() {
        // find field value for player's row
        let content = match radlci.get(&player.id) {
            Some(radlci) => radlci_to_string(radlci),
            None => "".to_string(),
        };
        
        let append = format!("<th>{}</th>", content);
        line = format!("{}{}", line,append);
    }
    line = format!("<tr>{}</tr>", line);
    table = format!("{}{}", table, line);
    // generate table rows 
    for index in 0..rounds {
        let mut line = String::from("");
        for player in players.iter() {
            // find field value for player's row
            
            let content = match score.get(&player.id) {
                Some(score) => match score[index as usize] {
                    Some(val) => {
                        let mut class= "".to_string();
                        let (_, min, max) = match sum_by_player.get(&player.id) {
                            Some(data) => data,
                            None => &(0, 0, 0),
                        };
                        if val == *min {
                            class = "class='smallest'".to_string();
                        }
                        if val == *max {
                            class = "class='biggest'".to_string();
                        }
                        let aditional_markers = get_aditional_markers(
                            global_player_attributes,
                            &player.id,
                            &index,
                        );

                        format!("<td {}>{} {}</td>", class, val, aditional_markers)
                    },
                    None => "<td></td>".to_string(),
                },
                None => "<td>Missing</td>".to_string(),
            };
            line = format!("{}{}", line, content);
        }
        table = format!("{}<tr>{}</tr>", table, line);
    }
    // final score
    let mut line = "".to_string();
    for player in players.iter() {
        // find field value for player's row
        let content = match sum_by_player.get(&player.id) {
            Some((sum, _, _)) => sum.to_string(),
            None => "".to_string(),
        };
        
        let append = format!("<th>{}</th>", content);
        line = format!("{}{}", line,append);
    }
    line = format!("<tr>{}</tr>", line);
    table = format!("{}{}", table, line);

    format!("{}{}{}", get_html_head(), table, get_html_tail())

}

fn get_aditional_markers(
    global_player_attributes: &mut HashMap<String, Vec<Option<Vec<TarokPlayerInput>>>>,
    player_id: &String,
    round: &i32,
) -> String {
    let atrs = match global_player_attributes.get(player_id) {
        Some(att) => att,
        None => return "".to_string(),
    };
    if *round >= atrs.len() as i32 {
        return "".to_string();
    }
    let round_atrs = match &atrs[*round as usize] {
        Some(att) => att,
        None => return "".to_string(),
    };
    let markers = round_atrs
        .iter()
        .map(|x| tarok_player_input_to_string(x))
        .collect::<Vec<String>>()
        .join("");
    format!("{}", markers)
}

fn tarok_player_input_to_string(x: &TarokPlayerInput) -> String {
    match x {
        TarokPlayerInput::PlayerDiff(_) => "".to_string(),
        TarokPlayerInput::PlayerAttribute(a) => match a {
            super::enums::TarokPlayerAttibute::M => "<i title='Mond snipe' class='fas fa-crosshairs'></i>".to_string(),
            super::enums::TarokPlayerAttibute::R => "<i title='Renons' class='fas fa-hand-middle-finger'></i>".to_string(),
            super::enums::TarokPlayerAttibute::T => "<i title='Renons' class='fas fa-users-slash'></i>".to_string(),
            super::enums::TarokPlayerAttibute::Ig => "<i title='Renons' class='fas fa-dice'></i>".to_string(),
            super::enums::TarokPlayerAttibute::Sl => "<i class='fal fa-truck-container'></i>".to_string(),
        },
    }
}

fn radlci_to_string(radlci: &[Radlc]) -> String {
    let mut out = "".to_string();
    for radl in radlci.iter() {
        if let Radlc::Avalible = radl {
            out = format!("{} O",out);
        } else {
            out = format!("{} Ø",out);
        }
    }
    out
}

fn get_html_tail() -> String {
    "</table>
    <i title='Renons' class='fa-solid fa-truck-tow'></i>
    <i class='fa-solid fa-truck-tow'></i>
    <i class='fal fa-truck-container'></i>
    </body>
    </html>".to_string()
}

fn get_html_head() -> String {
    "<!DOCTYPE html><html lang='en'><head><link rel='stylesheet' href='https://pro.fontawesome.com/releases/v5.10.0/css/all.css'
    integrity='sha384-AYmEC3Yw5cVb3ZcuHtOA93w35dYTsvhLPVnYs9eStHfGJvOvKxVfELGroGkvsg+p' crossorigin='anonymous' />
    <meta charset='UTF-8'><meta http-equiv='X-UA-Compatible' content='IE=edge'><meta name='viewport' content='width=device-width, initial-scale=1.0'>
    <title>Document</title></head><body><style>
    table{width: 100%;text-align: center;}tr:nth-child(2n+1) {background-color: rgb(229 228 228)}.biggest {color: 
    green;}.smallest {color: red}</style><table>".to_string()
}