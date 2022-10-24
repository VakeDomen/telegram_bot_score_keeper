use std::collections::HashMap;

use crate::models::user::User;

pub fn build_score_table_html(players: Vec<User>, score_table: HashMap<String, Vec<Option<i32>>>, rounds: i32) -> String {
    let mut table = String::from("");
    // generate table header
    for player in players.iter() {
        let append = format!("<th>{}</th>", player.name);
        table = format!("{}{}", table,append)
    }
    table = format!("<tr>{}</tr>", table);
    // generate table rows 
    for index in 0..rounds {
        let mut line = String::from("");
        for player in players.iter() {
            // find field value for player's row
            let content = match score_table.get(&player.id) {
                Some(score) => match score[index as usize] {
                    Some(val) => format!("{}", val),
                    None => "".to_string(),
                },
                None => "Missing".to_string(),
            };
            
            let append = format!("<td>{}</td>", content);
            line = format!("{}{}", line,append);
        }
        table = format!("{}<tr>{}</tr>", table, line);
    }
    format!("{}{}{}", get_html_head(), table, get_html_tail())

}

fn get_html_tail() -> String {
    "</table>
    </body>
    </html>".to_string()
}

fn get_html_head() -> String {
    "<!DOCTYPE html>
    <html lang='en'>
    <head>
            <meta charset='UTF-8'>
            <meta http-equiv='X-UA-Compatible' content='IE=edge'>
            <meta name='viewport' content='width=device-width, initial-scale=1.0'>
            <title>Document</title>
    </head>
    <body>
            <style>
                    table{
                            width: 100%;
                            text-align: center;
                    }
                    tr:nth-child(2n) {color: rgb(128, 128, 128)}
                    th {
                            color: #a6acf3;
                    }
                    .biggest {
                            color: green;
                    }
                    .smallest {
                            color: red
                    }
                    tr {
                        border: 1px solid rgb(190, 190, 190);
                    }
            </style>
            <table>
            ".to_string()
}