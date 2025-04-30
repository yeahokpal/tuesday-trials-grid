use std::{env, error::Error, fs, io::Write};

use rusqlite::vtab::csvtab;

use crate::db_builder::DbBuilderState;

pub async fn get_sheets_data(state: &DbBuilderState) -> Result<(), Box<dyn Error>> {
    let sheet_id = env::var("SHEET_ID")?;
    csvtab::load_module(&state.sql)?;
    for sheet in vec!["Game Deduplication", "Player Data", "Stream Data", "Stream Rename", "Stream Player Rename"] {
        let urlify = sheet.replace(' ', "%20");
        let data = state.client.get(format!("https://docs.google.com/spreadsheets/d/{sheet_id}/gviz/tq?tqx=out:csv&sheet={urlify}"))
            .send().await?.bytes().await?;
        let db_name = sheet.replace(' ', "");
        fs::File::create(format!("{db_name}.csv"))?.write(&data)?;
        
        state.sql.execute(format!("DROP TABLE IF EXISTS temp.{db_name}").as_str(), [])?;
        state.sql.execute(format!("CREATE VIRTUAL TABLE temp.{db_name} USING csv(filename='{db_name}.csv', header=TRUE)").as_str(), [])?;
    }

    for stmt in vec![
    "UPDATE Event
    SET Name = dedup.[Rename To]
    FROM temp.GameDeduplication dedup
    WHERE Event.Name = dedup.Name
    AND dedup.[Rename To] > ''",

    "DELETE FROM Event WHERE Name IN (SELECT Name FROM temp.GameDeduplication WHERE [Delete] = 'Y')",

    "UPDATE Player
    SET Name = playerdat.[Rename To]
    FROM temp.PlayerData playerdat
    WHERE Player.ID = playerdat.ID AND playerdat.[Rename To] > ''",

    "DELETE FROM Stream",
    "INSERT INTO Stream
    SELECT DISTINCT Title, COALESCE(sr.[Rename To], sd.Game) AS Game
    , COALESCE(sp1.[Rename To], p1.[Rename To], sd.player1) AS Player1
    , COALESCE(sp2.[Rename To], p2.[Rename To], sd.player2) AS Player2
    FROM temp.StreamData sd
    LEFT JOIN temp.StreamRename sr ON sd.Game = sr.Game AND sr.[Rename To] > ''
    LEFT JOIN temp.StreamPlayerRename sp1 ON sp1.Name = sd.Player1 AND sp1.[Rename To] > ''
    LEFT JOIN temp.StreamPlayerRename sp2 ON sp2.Name = sd.Player2 AND sp2.[Rename To] > ''
    LEFT JOIN temp.PlayerData p1 ON p1.Name = sd.player1 AND p1.[Rename To] > ''
    LEFT JOIN temp.PlayerData p2 ON p2.Name = sd.player2 AND p2.[Rename To] > ''
    WHERE sd.Game > ''",

    "DELETE FROM Controller",
    "INSERT INTO Controller
    SELECT ID, TRIM(Controller) FROM temp.PlayerData
    WHERE Controller > ''",

    "DELETE FROM Character"
    ] {
        let res = state.sql.execute(stmt, [])?;
        dbg!(res);
    }

    let miscdata: Vec<String> = state.sql.prepare("SELECT name FROM pragma_table_info('PlayerData') WHERE cid > 5")?
        .query_map((), |row|row.get("name"))?
        .collect::<Result<_, _>>()?;

    for col in miscdata.iter().filter(|s|s.contains("Chars Played"))
    {
        let playerdata: Vec<(String, String)> = state.sql.prepare(format!("SELECT ID, [{col}] FROM temp.PlayerData WHERE [{col}] > ''").as_str())?
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
    SELECT ID, {cols} FROM temp.PlayerData
    WHERE {cols_where};
    ").as_str(), ())?;

    state.sql.cache_flush()?;

    Ok(())
}
    