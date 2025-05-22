use std::error::Error;

use db_builder::{build_db, build_last_trials, build_sheets_data, update_players};
use grid_builder::build_grid;
use reqwest::Client;
use rusqlite::Connection;
use sheets_data::get_sheets_data;

mod db_builder;
mod grid_builder;
mod queries;
mod sheets_data;

#[tokio::main]
async fn main() -> rusqlite::Result<()> {
    // let client = Client::new();
    // let sql = Connection::open("./db.sqlite").expect("failed to open db");
    // let res = stmt.query_map([], |res| res.get("ID").and_then(|i|Ok(i32::to_string(&i))))?;

    // let vec = res.map(|i| i.unwrap()).collect::<Vec<_>>();
    // dbg!(&vec);

    // match update_players(&client, &sql, &mut vec.iter()).await {
    // match build_last_trials().await {
    // match build_db(false).await {
    match build_sheets_data().await {
        Err(e)=>{dbg!(&e);},
        Ok(_) => {print!("success\n");}
    };
    // dbg!(build_grid());
    Ok(())
}