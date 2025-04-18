use std::{error::Error, fs, iter, ops::{Index, IndexMut}};

use rand::{distr::uniform::SampleRange, rngs::ThreadRng, Rng};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Var {
    var: String,
    query: Option<String>,
    values: Option<Vec<String>>,
    labels: Option<Vec<Option<String>>>,
}
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
    queries: Vec<Query>,
    vars: Vec<Var>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Grid {
    rows: Vec<Query>,
    columns: Vec<Query>,
    answers: [Vec<String>; 9]
}

fn get_random_queries(query_file: &QueryFile, sql: &Connection, rng: &mut ThreadRng, count: usize) -> Vec<Query> {
    let max = query_file.queries.iter().map(|q| q.odds).sum();
    iter::repeat_with(||{
        let mut iter = query_file.queries.iter();
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
            Some(opts[rng.random_range(0..opts.len())].iter().map(|v|(v.clone(), None)).collect::<Vec<(String, Option<String>)>>())
        ).or_else(||
            q.vars.as_ref().and_then(|vars|{Some(vars.iter().map(|var| {
                query_file.vars.iter().find(|v|&v.var == var)
                    .and_then(|v|
                        v.values.clone()
                        .or_else(||
                            Some(sql.prepare(v.query.as_ref()?.as_str())
                                .expect("sql err").query_map([], |row| row.get(0)).unwrap()
                                .into_iter()
                                .map(|r|r.unwrap()).collect())
                        )
                        .and_then(|a|
                            Some(a.into_iter().zip(v.labels.clone().into_iter().flatten().chain(iter::repeat(None))).collect())
                        )
                    )
                    .and_then(|arr: Vec<(String, Option<String>)>| {
                        let rand = rng.random_range(0..arr.len());
                        arr.into_iter().skip(rand).next()
                    })
                    .expect("aaa")
                }).collect()
            )})
        ) {
            for (i, (value, label)) in options.into_iter().enumerate() {
                let var = format!("[{}]", &q.vars.as_ref().unwrap()[i]);
                q.query = q.query.replace(&var, &value);
                q.label = q.label.replace(&var, label.as_ref().unwrap_or(&value));
            }
        }

        return q
    }).take(count).collect()
}

fn init_answers(sql: &Connection, grid: &mut Grid) {
    for (i, answer) in &mut grid.answers.iter_mut().enumerate() {
        let (row_q, col_q) = (&grid.rows[i % 3], &grid.columns[i / 3]);
        let mut stmt = sql.prepare(format!("{} INTERSECT {}", row_q.query, col_q.query).as_str()).expect("sql error");
        let mut res: Vec<String> = stmt.query_map([], |row|row.get(0)).expect("sql error").map(|r|r.unwrap()).collect();
        answer.append(&mut res);
    }
}

pub fn build_grid() -> Result<Grid, Box<dyn Error>> {
    let sql = Connection::open("./db.sqlite")?;
    let queries: QueryFile = toml::from_str(fs::read_to_string("./src/GridQueries.toml")?.as_str())?;
    let mut rng = rand::rng();
    let mut return_grid: Option<Grid> = None;

    while return_grid.is_none() {
        let mut grid: Grid = Grid {
            rows: get_random_queries(&queries, &sql, &mut rng, 3),
            columns: get_random_queries(&queries, &sql, &mut rng, 3),
            answers: Default::default()
        };
        init_answers(&sql, &mut grid);
        if grid.answers.iter().all(|a|a.len() > 0) {
            return_grid = Some(grid);
        }
    };

    return_grid.ok_or("Could not create grid".into())
}