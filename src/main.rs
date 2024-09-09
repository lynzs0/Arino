pub mod database;
pub mod cli_operations;
pub mod helper;

use cli_operations::user_input::{self, prompt};
use database::connection::get_connection;

fn main() {
    let path: String = "d:\\lyns0\\Dev\\Database\\project_kechi.db".to_string();
    let connection = get_connection(&path);
    
    println!("----Arino----");
    loop {
        let user_input = prompt("Command");
        user_input::match_commands(user_input, &connection);
    }
}
