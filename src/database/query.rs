pub mod dish_by_ingredients;

use std::collections::{HashMap, HashSet};
use rusqlite::{Connection, Result};
use crate::helper::calculate_mean;
use prettytable::{Cell, Row, Table};

use super::logic::filter_dishes_with_input_ingredients;
#[allow(dead_code)]

pub fn all_dish_names(conn: &Connection) -> Result<()> {
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

pub fn dish_by_ingredients(arg_vec: Vec<String>, conn: &Connection) -> Result<()> {
    let input_ingredients_vec = arg_vec;
    let mut input_ingredient_ids_vec: Vec<u32> = Vec::new(); 

    if input_ingredients_vec.is_empty() {
        eprintln!("No ingredient was input for dish query");
        return Ok(());
    }

    // get all ingredient id
    let mut select_ingredient_ids_stmt = conn.prepare("SELECT id FROM ingredients;")?;
    let all_ingredient_ids_set: HashSet<u32> = select_ingredient_ids_stmt
        .query_map([], |row| Ok(row.get::<_, u32>(0)?))?
        .map(|result| result.unwrap())
        .collect();

    // get input ingredient id
    for name in input_ingredients_vec {
        let mut select_ingredient_ids_by_name_stmt = conn.prepare("SELECT id FROM ingredients WHERE name = ?1;")?;
        let ingredient_ids_result: Result<u32> = select_ingredient_ids_by_name_stmt.query_row([&name], |row| row.get(0));
        match ingredient_ids_result {
            Ok(id) => {
                input_ingredient_ids_vec.push(id);
            },
            Err(e) => {
                eprintln!("Ingredient \"{}\" does not exist in database.", name);
                eprintln!("{}", e);
                return Ok(());
            }
        };
    }

    let input_ingredient_ids_set: HashSet<u32> = input_ingredient_ids_vec.into_iter().collect();

    let all_dish_recipes_map = get_all_recipes_map(&conn)?;

    let filtered_dish_ids_vec = filter_dishes_with_input_ingredients(&input_ingredient_ids_set, &all_ingredient_ids_set, &all_dish_recipes_map);
    
    let mut available_dishes: Vec<String> = Vec::new();

    for id in filtered_dish_ids_vec {
        let mut select_dish_names_by_id_stmt = conn.prepare("SELECT name FROM dishes WHERE id = ?1;")?;
        let dish_names: String = select_dish_names_by_id_stmt.query_row([id], |row| row.get(0))?;
        available_dishes.push(dish_names);
    }

    if !available_dishes.is_empty() {
        let mut table: Table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("Available Dish"),
        ]));

        for dish in available_dishes {
            table.add_row(Row::new(vec![
                Cell::new(&dish),
            ]));
        }

        table.printstd();
    } else {
        println!("No available dishes");
    }
    
    Ok(())
}

fn get_all_recipes_map(conn: &Connection) -> Result<HashMap<u32, Vec<u32>>> {
    let mut all_recipes_map: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut select_dish_ids_stmt = conn.prepare("SELECT id FROM dishes;")?;
    let dish_ids_vec: Vec<u32> = select_dish_ids_stmt
        .query_map([], |row| Ok(row.get::<_, u32>(0)?))?
        .map(|result| result.unwrap())
        .collect();

    for dish_id in dish_ids_vec {
        let mut select_recipe_ingredient_ids_stmt = conn.prepare("SELECT ingredient_id FROM recipes WHERE dish_id = ?1;")?;
        let ingredient_ids_vec: Vec<u32> = select_recipe_ingredient_ids_stmt
            .query_map([dish_id], |row| Ok(row.get::<_, u32>(0)?))?
            .map(|result| result.unwrap())
            .collect();
        all_recipes_map.insert(dish_id, ingredient_ids_vec);
    }

    Ok(all_recipes_map)
}

pub fn recipe_by_dish_name(arg_vec: Vec<String>, conn: &Connection) -> Result<()> {
    if arg_vec.is_empty() {
        eprintln!("No dish name input for recipe query");
        return Ok(());
    }

    let dish_name = &arg_vec[0];

    let mut select_dish_ids_by_name_stmt = conn.prepare("SELECT id FROM dishes WHERE name = ?1;")?;
    let dish_id_result: Result<u32> = select_dish_ids_by_name_stmt.query_row([dish_name], |row| row.get(0));

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
        Cell::new("Quantity"),
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

pub fn all_ingredients(conn: &Connection) -> Result<()> {
    let mut select_ingredients_stmt = conn.prepare("SELECT * FROM ingredients;")?;
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

    Ok(())
}