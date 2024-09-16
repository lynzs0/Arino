pub mod dish_by_ingredients;

use rusqlite::Result;
use crate::{cli_operations::user_input::prompt, helper::calculate_mean};
use prettytable::{Cell, Row, Table};

use super::get_connection;

pub fn all_dish_names() -> Result<()> {
    let conn = get_connection();
    let mut select_dish_names_stmt = conn.prepare("Select id, name FROM dishes")?;
    let names_iter = select_dish_names_stmt.query_map([], |row| {
        Ok((row.get::<_, u32>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut table: Table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("ID"),
        Cell::new("Name"),
    ]));

    for dish in names_iter {
        let (id, name) = dish?;

        table.add_row(Row::new(vec![
            Cell::new(&id.to_string()),
            Cell::new(&name),
        ]));
    }

    table.printstd();

    Ok(())
}

pub fn recipe_by_dish_name() -> Result<()> {
    let conn = get_connection();

    let dish_name = prompt("Dish name");
    
    if dish_name.trim().is_empty() {
        return Ok(());
    }

    let mut select_dish_ids_by_name_stmt = conn.prepare("SELECT id FROM dishes WHERE name = ?1;")?;
    let dish_id_result: Result<u32> = select_dish_ids_by_name_stmt.query_row([&dish_name], |row| row.get(0));

    let dish_id = match dish_id_result {
        Ok(id) => id,
        Err(e) => {
            eprint!("{e}");
            return Ok(());
        },
    };
    
    let mut select_recipe_ingredient_ids_stmt = conn.prepare("SELECT ingredient_id FROM recipes WHERE dish_id = ?1;")?;
    let ingredient_ids_iter = select_recipe_ingredient_ids_stmt.query_map([dish_id], |row| {
        Ok(row.get::<_, u32>(0)?)
    })?;

    let mut ingredient_names: Vec<String> = Vec::new();
    let mut ingredient_quantities: Vec<u32> = Vec::new();

    for ingredient_id in ingredient_ids_iter {
        let ingredient_id = ingredient_id?;
        let ingredient_name: String = conn.query_row(
            "SELECT name FROM ingredients WHERE id = ?1;",
            [ingredient_id],
            |row| row.get(0),
        )?;

        ingredient_names.push(ingredient_name);

        let ingredient_quantity: u32 = conn.query_row(
            "SELECT quantity FROM recipes WHERE dish_id = ?1 AND ingredient_id = ?2;",
            [dish_id, ingredient_id],
            |row| row.get(0),
        )?;
            
        ingredient_quantities.push(ingredient_quantity);
    }

    println!("{}", "-".repeat(50));
    println!("Recipe for {dish_name}:");
    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("Ingredient"),
        Cell::new("Quantity (normally g)"),
    ]));
    for (name, quantity) in ingredient_names.iter().zip(ingredient_quantities.iter()) {
        table.add_row(Row::new(vec![
            Cell::new(&name),
            Cell::new(&quantity.to_string()),
        ]));
    }
    table.printstd();

    Ok(())
}

pub fn all_ingredients() -> Result<()> {
    let conn = get_connection();

    let (category_name, category_id) = loop {
        let input_category_name = prompt("Category (all, vegetable, fruit, dairy, meat, condiment, grain)");
        if input_category_name.is_empty() {
            return Ok(());
        }

        if input_category_name == "all" {
            break (input_category_name, 0);
        }

        let retrieved_category_id: u32 = match conn.query_row("SELECT id FROM categories WHERE name = ?1;", [&input_category_name], |row| row.get(0)) {
            Ok(id) => id,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                eprintln!("Invalid category");
                continue;
            },
            Err(e) => {
                eprintln!("Error: {e}");
                continue;
            }
        };
        break (input_category_name, retrieved_category_id);
    };

    if category_id > 0 {
        let mut select_ingredients_stmt = conn.prepare("SELECT * FROM ingredients WHERE category_id = ?1;")?;
        let ingredients_iter = select_ingredients_stmt.query_map([category_id], |row| {
            Ok((row.get::<_, i32>(0)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?))
        })?;

        let mut table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("ID"),
            Cell::new(&format!("Name ({category_name})")),
            Cell::new("Lifespan"),
            Cell::new("Price"),
        ]));
    
        for ingredient in ingredients_iter {
            let (id, name, lifespan) = ingredient?;
            let mut price_query = conn.prepare("SELECT price from prices where ingredient_id = ?1;")?;
            let prices_iter = price_query.query_map([id], |row| {
                Ok(row.get::<_, f32>(0)?)
            })?;
    
            let mut prices: Vec<f32> = Vec::new();
    
            for price in prices_iter {
                prices.push(price?);
            }
    
            let mean_price = calculate_mean(prices);
    
            table.add_row(Row::new(vec![
                Cell::new(&id.to_string()),
                Cell::new(&name),
                Cell::new(&lifespan),
                Cell::new(&format!("${mean_price:.2}")),
            ]));
        }
    
        table.printstd();
    }
    else {
        let mut select_ingredients_stmt = conn.prepare("SELECT * FROM ingredients")?;
        let ingredients_iter = select_ingredients_stmt.query_map([], |row| {
            Ok((row.get::<_, i32>(0)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?))
        })?;

        let mut table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("ID"),
            Cell::new("Name"),
            Cell::new("Lifespan"),
            Cell::new("Price"),
        ]));
    
        for ingredient in ingredients_iter {
            let (id, name, lifespan) = ingredient?;
            let mut price_query = conn.prepare("SELECT price from prices where ingredient_id = ?1;")?;
            let prices_iter = price_query.query_map([id], |row| {
                Ok(row.get::<_, f32>(0)?)
            })?;
    
            let mut prices: Vec<f32> = Vec::new();
    
            for price in prices_iter {
                prices.push(price?);
            }
    
            let mean_price = calculate_mean(prices);
    
            table.add_row(Row::new(vec![
                Cell::new(&id.to_string()),
                Cell::new(&name),
                Cell::new(&lifespan),
                Cell::new(&format!("${mean_price:.2}")),
            ]));
        }
    
        table.printstd();
    }

    Ok(())
}