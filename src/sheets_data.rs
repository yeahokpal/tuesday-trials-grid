use std::{env, error::Error, fs, io::Write};

use rusqlite::{types::{FromSql, FromSqlResult, Value}, vtab::csvtab};
use serde::Serialize;

use crate::db_builder::DbBuilderState;

pub async fn get_sheets_data(state: &DbBuilderState) -> Result<(), Box<dyn Error>> {
    let sheet_id = env::var("SHEET_ID")?;
    csvtab::load_module(&state.sql)?;
    for sheet in vec!["Game Deduplication", "Player Data", "Stream Data", "Stream Rename", "Stream Player Rename"] {
        let urlify = sheet.replace(' ', "%20");
        let data = state.client.get(format!("https://docs.google.com/spreadsheets/d/{sheet_id}/gviz/tq?tqx=out:csv&sheet={urlify}"))
            .send().await?.bytes().await?;
        let db_name = sheet.replace(' ', "");
        let mut f = fs::File::create(format!("{db_name}.csv"))?;
        f.write(&data)?;
        f.flush()?;
        
        state.sql.execute(format!("DROP TABLE IF EXISTS temp.{db_name}").as_str(), [])?;
        state.sql.execute(format!("CREATE VIRTUAL TABLE temp.{db_name} USING csv(filename='{db_name}.csv', header=TRUE)").as_str(), [])?;
    }

    for stmt in vec![
    "DELETE FROM EventRename",
    "INSERT INTO EventRename
    SELECT Name, [Rename To], [Delete]
    FROM temp.GameDeduplication
    ",
    "UPDATE EventRename SET [Rename To] = Name WHERE [Rename To] = ''",
    "UPDATE Player SET DisplayName = Name",
    "UPDATE Player
    SET DisplayName = playerdat.[Rename To]
    FROM temp.PlayerData playerdat
    WHERE Player.ID = playerdat.ID AND playerdat.[Rename To] > ''",

    "DELETE FROM Stream",
    "INSERT INTO Stream
    SELECT DISTINCT Title, COALESCE(sr.[Rename To], sd.Game) AS Game
    , COALESCE(p1.DisplayName, sd.player1) AS Player1
    , COALESCE(p2.DisplayName, sd.player2) AS Player2
    FROM temp.StreamData sd
    LEFT JOIN temp.StreamRename sr ON sd.Game = sr.Game AND sr.[Rename To] > ''
    LEFT JOIN temp.StreamPlayerRename sp1 ON sp1.Name = sd.Player1
    LEFT JOIN temp.StreamPlayerRename sp2 ON sp2.Name = sd.Player2
    LEFT JOIN Player p1 ON p1.ID = sp1.ID
    LEFT JOIN Player p2 ON p2.ID = sp2.ID
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
    {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open("PlayerData.csv")?;
        file.write(&[b'\n'])?;
        let mut csv = csv::Writer::from_writer(file);
        let mut stmt = state.sql.prepare("
        SELECT p.ID, p.Name, (SELECT COUNT(*) FROM Standing WHERE PlayerID = p.ID) AS Count
        FROM Player p
        LEFT JOIN temp.PlayerData pd ON pd.ID = p.ID
        WHERE pd.ID IS NULL")?;
        let count = stmt.column_count();
        let mut rows = stmt.query([])?;
        while let Some(item) = rows.next()? {
            csv.write_record((0..count).map(|i|match item.get::<_, Value>(i).unwrap()
            {
                Value::Null => "".to_string(),
                Value::Integer(i) => i.to_string(),
                Value::Real(i) => i.to_string(),
                Value::Text(s) => s,
                Value::Blob(_) => "".to_string(),
            }))?;
        }
        csv.flush()?;
    }
    {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open("GameDeduplication.csv")?;
        file.write(&[b'\n'])?;
        let mut csv = csv::Writer::from_writer(file);
        let mut stmt = state.sql.prepare("
        SELECT e.Name, COUNT(*) AS Count, '' [Rename To], 'New' [Delete]
        FROM Event e
        LEFT JOIN temp.GameDeduplication g ON g.Name = e.Name
        WHERE g.Name IS NULL
        GROUP BY e.Name")?;
        let count = stmt.column_count();
        let mut rows = stmt.query([])?;
        while let Some(item) = rows.next()? {
            csv.write_record((0..count).map(|i|match item.get::<_, Value>(i).unwrap()
            {
                Value::Null => "".to_string(),
                Value::Integer(i) => i.to_string(),
                Value::Real(i) => i.to_string(),
                Value::Text(s) => s,
                Value::Blob(_) => "".to_string(),
            }))?;
        }
        csv.flush()?;
    }

    Ok(())
}
    