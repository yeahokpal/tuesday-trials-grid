use std::{env, error::Error, fs, io::Write};

use rusqlite::vtab::csvtab;

use crate::db_builder::DbBuilderState;

pub async fn get_sheets_data(state: &DbBuilderState) -> Result<(), Box<dyn Error>> {
    let sheet_id = env::var("SHEET_ID")?;
    let dedup = state.client.get(format!("https://docs.google.com/spreadsheets/d/{sheet_id}/gviz/tq?tqx=out:csv&sheet=Game%20Deduplication"))
        .send().await?.bytes().await?;
    let playerdata = state.client.get(format!("https://docs.google.com/spreadsheets/d/{sheet_id}/gviz/tq?tqx=out:csv&sheet=Player%20Data"))
        .send().await?.bytes().await?;
    let streamdata = state.client.get(format!("https://docs.google.com/spreadsheets/d/{sheet_id}/gviz/tq?tqx=out:csv&sheet=Stream%20Data"))
        .send().await?.bytes().await?;

    fs::File::create("dedup.csv")?.write(&dedup)?;
    fs::File::create("playerdata.csv")?.write(&playerdata)?;
    fs::File::create("streamdata.csv")?.write(&streamdata)?;
    csvtab::load_module(&state.sql)?;


    for stmt in vec![
    "DROP TABLE IF EXISTS temp.dedup",
    "DROP TABLE IF EXISTS temp.playerdat",
    "DROP TABLE IF EXISTS temp.streamdata",
    "CREATE VIRTUAL TABLE temp.dedup USING csv(filename='dedup.csv', header=TRUE)",
    "CREATE VIRTUAL TABLE temp.playerdat USING csv(filename='playerdata.csv', header=TRUE)",
    "CREATE VIRTUAL TABLE temp.streamdata USING csv(filename='streamdata.csv', header=TRUE)",

    "UPDATE Event
    SET Name = dedup.[Rename To]
    FROM temp.dedup dedup
    WHERE Event.Name = dedup.Name
    AND dedup.[Rename To] > ''",

    "DELETE FROM Event WHERE Name IN (SELECT Name FROM temp.dedup WHERE [Delete] = 'Y')",

    "UPDATE Player
    SET Name = playerdat.[Rename To]
    FROM temp.playerdat playerdat
    WHERE Player.ID = playerdat.ID AND playerdat.[Rename To] > ''",

    "DELETE FROM Stream",
    "DELETE FROM Character",
    "INSERT INTO Stream
    SELECT * FROM temp.streamdata",
    "DELETE FROM Controller",
    "INSERT INTO Controller
    SELECT ID, TRIM(Controller) FROM temp.playerdat
    WHERE Controller > ''"
    ] {
        let res = state.sql.execute(stmt, [])?;
        dbg!(res);
    }

    let miscdata: Vec<String> = state.sql.prepare("SELECT name FROM pragma_table_info('playerdat') WHERE cid > 5")?
        .query_map((), |row|row.get("name"))?
        .collect::<Result<_, _>>()?;

    for col in miscdata.iter().filter(|s|s.contains("Chars Played"))
    {
        let playerdata: Vec<(String, String)> = state.sql.prepare(format!("SELECT ID, [{col}] FROM playerdat WHERE [{col}] > ''").as_str())?
            .query_map((), |row|Ok((row.get("ID")?, row.get(col.as_str())?)))?
            .collect::<Result<_, _>>()?;
        let game_name = col.replace(" Chars Played", "");
        let insert = playerdata.iter().flat_map(|p| p.1.split(", ")
            .map(|s|s.trim())
            .map(|s|format!("({p}, '{game_name}', '{s}')", p = p.0)))
            .collect::<Vec<_>>().join(",");
        state.sql.execute(&format!("INSERT INTO Character
            VALUES {insert}"), ())?;
    }

    let cols = miscdata.iter().filter(|s|!s.contains("Chars Played"))
        .map(|s|format!("TRIM([{s}])"))
        .collect::<Vec<_>>().join(",");
    let cols_dec = miscdata.iter().filter(|s|!s.contains("Chars Played"))
        .map(|s|format!("\"{s}\" TEXT"))
        .collect::<Vec<_>>().join(",");
    let cols_where = miscdata.iter().filter(|s|!s.contains("Chars Played"))
        .map(|s|format!("[{s}] > ''"))
        .collect::<Vec<_>>().join(" OR ");

    state.sql.execute("DROP TABLE IF EXISTS MiscData", ())?;
    state.sql.execute(format!("CREATE TABLE MiscData (ID INTEGER, {cols_dec})").as_str(), ())?;

    state.sql.execute(format!("
    INSERT INTO MiscData
    SELECT ID, {cols} FROM playerdat
    WHERE {cols_where};
    ").as_str(), ())?;

    state.sql.cache_flush()?;

    Ok(())
}
    