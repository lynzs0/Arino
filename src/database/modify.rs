use rusqlite::Result;

use crate::{cli_operations::user_input::prompt, database::cloud::sync};

use super::{cloud::{fetch, has_internet_access, Database}, get, get_connection, query};

pub async fn ingredient() -> Result<()> {
    if !has_internet_access().await {
        return Ok(());
    }
    
    match fetch(Database::Main).await {
        Ok(_) => {},
        Err(e) => {
            eprintln!("{e}");
            return Ok(());
        },
    }

    let conn = get_connection();

    let ingredient_id = match get::ingredient_id(&conn) {
        Some(id) => id,
        None => return Ok(()),
    };

    let new_name = prompt("New Name");

    if !new_name.is_empty() {
        let mut update_name_stmt = conn.prepare("UPDATE ingredients SET name = ?1 WHERE id = ?2")?;
        update_name_stmt.execute((&new_name, &ingredient_id))?;
    }

    let new_lifespan = prompt("New Lifespan");

    if !new_lifespan.is_empty() {
        let mut update_lifespan_stmt = conn.prepare("UPDATE ingredients SET name = ?1 WHERE id = ?2")?;
        update_lifespan_stmt.execute((&new_lifespan, &ingredient_id))?;
    }

    match get::category_name_and_id(&conn) {
        Some((_, category_id)) => {
            let mut update_category_stmt = conn.prepare("UPDATE ingredients SET category_id = ?1 WHERE id = ?2")?;
            update_category_stmt.execute((category_id, &ingredient_id))?;
        }
        None => {},
    }

    println!("Ingredient Updated");
    match query::specific_ingredient(ingredient_id) {
        Ok(_) => {},
        Err(e) => eprintln!("Error: {e}"),
    }

    match sync().await {
        Ok(_) => {},
        Err(e) => {
            eprintln!("{e}");
            return Ok(());
        },
    }

    Ok(())
}