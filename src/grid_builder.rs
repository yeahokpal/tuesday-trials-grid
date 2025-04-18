use std::{error::Error, fs, ops::{Index, IndexMut}};

use rand::{distr::uniform::SampleRange, rngs::ThreadRng, Rng};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Query {
    label: String,
    query: String,
    odds: i32,
    vars: Option<Vec<String>>,
    options: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct QueryFile {
    queries: Vec<Query>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Grid {
    rows: Vec<Query>,
    columns: Vec<Query>,
    answers: [Vec<String>; 9]
}

fn get_random_queries(queries: &Vec<Query>, sql: &Connection, rng: &mut ThreadRng, count: i32) -> Vec<Query> {
    let max = queries.iter().map(|q| q.odds).sum();
    (0..count).map(|_| {
        let mut iter = queries.iter();
        let mut target = rng.random_range(0..max);
        let mut oq: Option<&Query> = None;
        while let Some(qr) = iter.next() {
            target -= qr.odds;
            if target < 0 {
                oq = Some(qr);
                break;
            }
        }
        let mut q = oq.unwrap().clone();


        if let Some(options) = q.options.as_ref().and_then(|opts|
            Some(opts[rng.random_range(0..opts.len())].iter().map(|v|(v.to_string(), None)).collect::<Vec<(String, Option<String>)>>())
        ).or_else(||
            q.vars.as_ref().and_then(|vars|{Some(vars.iter().map(|v| {
                let arr = match v.as_str() {
                "year" => sql.prepare("SELECT DISTINCT strftime('%Y', StartAt, 'unixepoch') FROM Tournament")
                            .expect("sql err").query_map([], |row| row.get(0)).unwrap()
                            .into_iter()
                            .map(|r|(r.unwrap(), None)).collect(),
                "above/below" => vec![("above".to_string(), Some(">".to_string())), ("below".to_string(), Some("<".to_string()))],
                "placement" => vec![("Top 3".to_string(), Some("3".to_string())), ("Top 8".to_string(), Some("8".to_string()))],
                "game" => sql.prepare("SELECT DISTINCT Name FROM Event")
                            .expect("sql err").query_map([], |row| row.get(0)).unwrap()
                            .into_iter()
                            .map(|r|(r.unwrap(), None)).collect(),
                "miscdata" => vec!["TRULY Esports", "SCUM Esports", "Quiznos Esports", "Has Had A Trials Named After Them"]
                            .iter().map(|s| (s.to_string(), None)).collect(),
                _ => vec![]
                };
                let rand = rng.random_range(0..arr.len());
                arr.into_iter().skip(rand).next().expect("random!")
            }).collect()
            )})
        ) {
            for (i, (label, query)) in options.into_iter().enumerate() {
                let var = format!("[{}]", &q.vars.as_ref().unwrap()[i]);
                q.label = q.label.replace(&var, &label);
                q.query = q.query.replace(&var, &query.unwrap_or(label));
            }
        }

        return q
    }).collect()
}

fn init_answers(sql: &Connection, grid: &mut Grid) {
    for (i, answer) in &mut grid.answers.iter_mut().enumerate() {
        let (rowQ, colQ) = (&grid.rows[i % 3], &grid.columns[i / 3]);
        let mut stmt = sql.prepare(format!("{} INTERSECT {}", rowQ.query, colQ.query).as_str()).expect("sql error");
        let mut res: Vec<String> = stmt.query_map([], |row|row.get(0)).expect("sql error").map(|r|r.unwrap()).collect();
        answer.append(&mut res);
    }
}

pub fn build_grid() -> Result<Grid, Box<dyn Error>> {
    let sql = Connection::open("./db.sqlite")?;
    let queries: QueryFile = toml::from_str(fs::read_to_string("./src/GridQueries.toml")?.as_str())?;
    let mut rng = rand::rng();
    let mut returnGrid: Option<Grid> = None;

    while returnGrid.is_none() {
        let mut grid: Grid = Grid {
            rows: get_random_queries(&queries.queries, &sql, &mut rng, 3),
            columns: get_random_queries(&queries.queries, &sql, &mut rng, 3),
            answers: Default::default()
        };
        init_answers(&sql, &mut grid);
        if grid.answers.iter().all(|a|a.len() > 0) {
            returnGrid = Some(grid);
        }
    };

    returnGrid.ok_or("Could not create grid".into())
}