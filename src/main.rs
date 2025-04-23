use std::error::Error;

use db_builder::{build_db, build_last_trials, update_players};
use grid_builder::build_grid;
use reqwest::Client;
use rusqlite::Connection;

mod db_builder;
mod grid_builder;
mod queries;

#[tokio::main]
async fn main() -> rusqlite::Result<()> {
    let client = Client::new();
    let sql = Connection::open("./db.sqlite").expect("failed to open db");
    let mut stmt = sql.prepare("select * from (select *, row_number() over (partition by p.name order by (select count(*) from standing where playerid = p.id) desc) rnk FROM Player p) where rnk = 1 and ProfileUrl is null ORDER BY RANDOM()")?;
    let res = stmt.query_map([], |res| res.get("ID").and_then(|i|Ok(i32::to_string(&i))))?;

    let vec = res.map(|i| i.unwrap()).collect::<Vec<_>>();
    dbg!(&vec);

    match update_players(&client, &sql, &mut vec.iter()).await {
    // match build_last_trials().await {
        Err(e)=>{dbg!(&e);},
        Ok(_) => {print!("success\n");}
    };
    // dbg!(build_grid());
    Ok(())
}