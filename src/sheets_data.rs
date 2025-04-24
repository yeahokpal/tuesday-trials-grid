use std::{env, error::Error, fs, io::Write};

use rusqlite::vtab::csvtab;

use crate::db_builder::DbBuilderState;

pub async fn get_sheets_data(state: &DbBuilderState) -> Result<(), Box<dyn Error>> {
    let sheet_id = env::var("SHEET_ID")?;
    let dedup = state.client.get(format!("https://docs.google.com/spreadsheets/d/{sheet_id}/gviz/tq?tqx=out:csv&sheet=Game%20Deduplication"))
        .send().await?.bytes().await?;
    let playerdata = state.client.get(format!("https://docs.google.com/spreadsheets/d/{sheet_id}/gviz/tq?tqx=out:csv&sheet=Player%20Data"))
        .send().await?.bytes().await?;

    fs::File::create("dedup.csv")?.write(&dedup)?;
    fs::File::create("playerdata.csv")?.write(&playerdata)?;
    csvtab::load_module(&state.sql)?;


    for stmt in vec![
    "DROP TABLE IF EXISTS dedup",
    "DROP TABLE IF EXISTS playerdat",
    "CREATE VIRTUAL TABLE dedup USING csv(filename='dedup.csv', header=TRUE)",
    "CREATE VIRTUAL TABLE playerdat USING csv(filename='playerdata.csv', header=TRUE)",
    
    "UPDATE Event
    SET Name = dedup.[Rename To]
    FROM dedup
    WHERE Event.Name = dedup.Name
    AND dedup.[Rename To] > ''",

    "DELETE FROM Event WHERE Name IN (SELECT Name FROM dedup WHERE [Delete] = 'Y')",

    "UPDATE Player
    SET Name = playerdat.[Rename To]
    FROM playerdat
    WHERE Player.ID = playerdat.ID AND playerdat.[Rename To] > ''",

    ] {
        let res = state.sql.execute(stmt, [])?;
        dbg!(res);
    }

    state.sql.cache_flush()?;

    Ok(())
}
    