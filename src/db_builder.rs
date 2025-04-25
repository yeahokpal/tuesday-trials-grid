use std::{collections::{HashMap, HashSet}, error::Error, fmt::Debug, time::Duration};

use futures::future::try_join_all;
use graphql_client::{GraphQLQuery, Response};
use crate::{queries::{get_participants, get_player::{self}, get_sets, get_tournament, get_tournaments, GetParticipants, GetPlayer, GetSets, GetTournament, GetTournaments, Tournament}, sheets_data::get_sheets_data};
use reqwest::Client;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

async fn make_request<V: Serialize, D: for<'a> Deserialize<'a>>(client: &Client, query: &V) -> Result<Response<D>, Box<dyn Error>>
    where V: Debug {
    let api_key = std::env::var("API_KEY")?;

    Ok(client.post("https://api.start.gg/gql/alpha")
        .bearer_auth(api_key)
        .json(query).send().await?
        .json().await?)
}

pub struct DbBuilderState {
    pub client: Client,
    pub sql: Connection,
    players: HashSet<String>,
}

pub async fn build_db(force_update: bool) -> Result<(), Box<dyn Error>> {

    let mut state = DbBuilderState {
        client: Client::new(),
        sql: Connection::open("./db.sqlite")?,
        players: HashSet::new(),
    };

    return get_sheets_data(&state).await;

    let tournaments = get_paged_query(
    async |page| { make_request(&state.client, &GetTournaments::build_query(get_tournaments::Variables {page: page})).await },
    |res: &get_tournaments::ResponseData| -> Option<i64> { res.tournaments.as_ref()?.page_info.as_ref()?.total_pages },
    &|res: get_tournaments::ResponseData| { res.tournaments?.nodes }).await
    .expect("tournaments query");

    for tournament in tournaments.into_iter()
    .filter_map(|x| x)
    .filter(|t| t.name.as_ref().unwrap().contains("Trial"))
    {
        dbg!(&tournament.name);
        update_tournament(&mut state, Tournament::try_from(tournament)?, force_update).await?;
    }
    update_players(&state.client, &state.sql).await?;

    Ok(())
}

pub async fn build_last_trials() -> Result<(), Box<dyn Error>> {
    let mut state = DbBuilderState {
        client: Client::new(),
        sql: Connection::open("./db.sqlite")?,
        players: HashSet::new()
    };
    let tournament = Tournament::try_from(make_request::<_, get_tournament::ResponseData>(&state.client, &GetTournament::build_query(get_tournament::Variables {slug: "trials".to_string()})).await?.data.expect("no data").tournament.expect("no data"))?;

    update_tournament(&mut state, tournament, true).await?;
    update_players(&state.client, &state.sql).await?;

    Ok(())
}


pub async fn update_players<'a>(client: &Client, sql: &Connection) -> Result<(), Box<dyn Error>> {
    let mut stmt = sql.prepare("
    select * 
    from (select *, row_number() over (partition by p.name order by (select count(*) from standing where playerid = p.id) desc) rnk FROM Player p) r
    JOIN Player p ON p.id = r.id
    where rnk = 1 AND p.Main IS NULL")?;
    for res in stmt.query_map((), |res| res.get("ID").and_then(|i|Ok(i32::to_string(&i))))? {
        match res {
        Ok(id) => {
            dbg!(&id);
            sleep(Duration::from_millis(1000)).await;
            let res = make_request::<_, get_player::ResponseData>(client, &GetPlayer::build_query(get_player::Variables{id: id.clone()})).await?;
            if let Some(url) = res.data.and_then(|d|d.player)
                .and_then(|p|p.user)
                .and_then(|u|u.images)
                .into_iter().flatten().collect::<Option<Vec<_>>>()
                .and_then(|images|images.get(0)?.url.clone())
            {
                sql.execute("UPDATE Player Set ProfileUrl = ?1 WHERE ID = ?2", (url, id))?;
            }
        }
        Err(e) => {return Err(e.into())}
        }
    }

    sql.execute("UPDATE Player SET Main = 1
    FROM (select *, row_number() over (partition by p.name order by (select count(*) from standing where playerid = p.id) desc) rnk FROM Player p) r
    where r.id = Player.id AND r.rnk = 1 AND Player.Main IS NULL
    ", ())?;

    Ok(())
}

async fn update_tournament(state: &mut DbBuilderState, tournament: Tournament, force_update: bool) -> Result<(), Box<dyn Error>>{
    let id = &tournament.id;
    let sql = &state.sql;
    let client = &state.client;
    let mut entrants: HashMap<String, String> = HashMap::new();

    let rows = sql.execute("INSERT INTO Tournament VALUES (?1, ?2, ?3) ON CONFLICT DO NOTHING", (id, &tournament.name, &tournament.start_at))?;
    if rows == 0 && !force_update {
        return Ok(());
    }
    dbg!("test1");
    for event in tournament.events.iter().flatten() {
        sql.execute("INSERT INTO Event VALUES (?1, ?2, ?3) ON CONFLICT DO NOTHING", (&event.id, &event.name, id))
        .expect("Event insert");
    }
    sleep(Duration::from_millis(300)).await;
    let participants = get_paged_query(
        async |page| { make_request(&client, &GetParticipants::build_query(get_participants::Variables {id: id.clone(), page, page_size: 60})).await },
        |res: &get_participants::ResponseData| -> Option<i64> { res.tournament.as_ref()?.participants.as_ref()?.page_info.as_ref()?.total_pages },
        &|res: get_participants::ResponseData| { res.tournament?.participants?.nodes }).await
        .expect("participants query");
    
    for participant in participants.into_iter().filter_map(|x| x) {
        let pid = participant.player.as_ref().and_then(|p| p.id.clone()).expect("Missing id");
        if !state.players.contains(&pid) {
            sql.execute("INSERT INTO Player(ID, Name) VALUES (?1, ?2) ON CONFLICT DO NOTHING", (&pid, participant.player.as_ref().and_then(|p|p.gamer_tag.as_ref()).expect("Missing id")))
            .expect("Player insert");
            state.players.insert(pid.clone());
        }
        for entrant in participant.entrants.into_iter().flatten().filter_map(|x| x) {
            entrants.insert(entrant.id.as_ref().expect("").clone(), pid.clone());
            if entrant.standing.as_ref().and_then(|f|f.is_final).unwrap_or(false) {
                sql.execute("INSERT INTO Standing VALUES (?1, ?2, ?3) ON CONFLICT DO UPDATE SET standing=excluded.standing", 
                    (&pid, 
                        entrant.event.expect("").id.expect(""), 
                        entrant.standing.unwrap().placement.unwrap()
                    )).expect("Standing insert");
            }
        }
    }
    let arr : Vec<String> = tournament.events.iter().flatten().map(|e| e.id.clone()).collect();

    for next_ten in arr.chunks(1) {
        let events = get_paged_query(async |page| { make_request(&client, &GetSets::build_query(get_sets::Variables { id: id.clone(), events: next_ten.iter().cloned().collect(), page, page_size: 60})).await },
        |res: &get_sets::ResponseData| -> Option<i64> { res.tournament.as_ref()?.events.as_ref()?.into_iter().flat_map(|e| { e.as_ref().expect("").sets.iter().map(|s| s.page_info.as_ref()?.total_pages) }).max().unwrap() },
        &|res: get_sets::ResponseData| { res.tournament?.events }).await
        .expect("sets query");
        sleep(Duration::from_millis(700)).await;

        for event in events.into_iter().filter_map(|e| e) {
            for set in event.sets.unwrap().nodes.unwrap().into_iter().filter_map(|s| s) {
                if let (Some(completed_at), started_at) = (set.completed_at, set.started_at) {
                if let Some(slots) = set.slots {
                if let (Some(slot1), Some(slot2)) = (&slots[0], &slots[1]) {
                if let (Some(placement1), Some(placement2)) = (slot1.standing.as_ref().and_then(|st|st.placement), slot2.standing.as_ref().and_then(|st|st.placement)) {
                if let (Some(winner), Some(loser)) = match placement1 - placement2 {
                        i if i < 0 => (Some(slot1), Some(slot2)),
                        i if i > 0 => (Some(slot2), Some(slot1)),
                        _ => (None, None) } {
                if let (Some(winner_id), Some(loser_id)) = (winner.entrant.as_ref().and_then(|e| e.id.as_ref()), loser.entrant.as_ref().and_then(|e| e.id.as_ref())) {
                if let (Some(winner_player_id), Some(loser_player_id)) = (entrants.get(winner_id), entrants.get(loser_id)) {
                let (winner_score, loser_score) = (winner.standing.as_ref().and_then(|st|st.stats.as_ref()).and_then(|s|s.score.as_ref()).and_then(|sc|sc.value), loser.standing.as_ref().and_then(|st|st.stats.as_ref()).and_then(|s|s.score.as_ref()).and_then(|sc|sc.value));
                    sql.execute("
INSERT INTO SetResult
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
ON CONFLICT DO UPDATE
SET eventID = excluded.eventID
, winnerID = excluded.winnerID
, loserID = excluded.loserID
, winnerScore = excluded.winnerScore
, loserScore = excluded.loserScore
, duration = excluded.duration", (&set.id, &event.id, winner_player_id, loser_player_id, winner_score, loser_score, started_at.and_then(|s| Some(completed_at - s))))?;
                }}}}}}}
            }
        }
    }
    Ok(())
}


async fn get_paged_query<Q, QR, PageCount, ResultsFunc, R>(query: Q, page_count: PageCount, results: &ResultsFunc) -> Result<Vec<R>, Box<dyn Error>>
    where 
        Q: AsyncFn(i64) -> Result<Response<QR>, Box<dyn Error>>,
        PageCount: Fn(&QR) -> Option<i64>, 
        ResultsFunc: Fn(QR) -> Option<Vec<R>> {
    let qr = query(1).await?;
    if let Some(err) = qr.errors {
        return Err(err[0].message.clone().into());
    }
    if let Some(data) = qr.data {
        let pages = page_count(&data).unwrap_or(0);

        return match results(data) {
            Some(r) if pages > 1 => Ok(r
                        .into_iter()
                        .chain(try_join_all(
                            (2..pages+1).map(async |page| { 
                                query(page)
                                .await?
                                .data
                                .and_then(results)
                                .ok_or::<Box<dyn Error>>("No data in page".into())
                            })
                        ).await?
                        .into_iter().flatten()).collect::<Vec<R>>()),
            Some(r) => Ok(r),
            None => Err("No data".into()),
        }
    }
    return Ok(Vec::new());
}