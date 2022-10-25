use std::collections::HashMap;

use crate::models::user::User;

use super::enums::Radlc;

pub fn build_score_table_html(
    players: &[User], 
    score: &HashMap<String, Vec<Option<i32>>>, 
    rounds: i32, 
    sum_by_player: HashMap<String, (i32, i32, i32)>,
    radlci: &HashMap<String, Vec<Radlc>>,
) -> String {
    println!("SCORE: {:#?}", score);
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
                        format!("<td {}>{}</td>", class, val)
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
    </body>
    </html>".to_string()
}

fn get_html_head() -> String {
    "<!DOCTYPE html><html lang='en'><head><meta charset='UTF-8'><meta http-equiv='X-UA-Compatible' content='IE=edge'>
    <meta name='viewport' content='width=device-width, initial-scale=1.0'><title>Document</title></head><body><style>
    table{width: 100%;text-align: center;}tr:nth-child(2n) {color: rgb(128, 128, 128)}th {color: #a6acf3;}.biggest {
    color: green;}.smallest {color: red}td,th {border: 1px solid rgb(190, 190, 190);}</style><table>".to_string()
}