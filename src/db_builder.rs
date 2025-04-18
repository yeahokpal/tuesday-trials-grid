use std::{collections::{HashMap, HashSet}, error::Error, fmt::Debug, hash::Hash, time::Duration};

use graphql_client::{GraphQLQuery, Response};
use reqwest::Client;
use rusqlite::Connection;
use serde::{de::Visitor, Deserialize, Serialize};
use futures::future::{join_all, try_join_all};
use tokio::time::sleep;

type Timestamp = i64;

#[derive(GraphQLQuery)]
#[graphql(schema_path = "schema.graphql", query_path = "src/queries/GetSets.graphql", variables_derives = "Debug")]
pub struct GetSets;

#[derive(GraphQLQuery)]
#[graphql(schema_path = "schema.graphql", query_path = "src/queries/GetTournaments.graphql", variables_derives = "Debug")]
pub struct GetTournaments;

#[derive(GraphQLQuery)]
#[graphql(schema_path = "schema.graphql", query_path = "src/queries/GetParticipants.graphql", variables_derives = "Debug")]
pub struct GetParticipants;

async fn make_request<V: Serialize, D: for<'a> Deserialize<'a>>(client: &Client, query: &V) -> Result<Response<D>, Box<dyn Error>>
    where V: Debug {
    let api_key = std::env::var("API_KEY")?;

    Ok(client.post("https://api.start.gg/gql/alpha")
        .bearer_auth(api_key)
        .json(query).send().await?
        .json().await?)
}

pub async fn build_db() -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let sql = Connection::open("./db.sqlite")?;

    let mut players: HashSet<String> = HashSet::new();
    let mut entrants: HashMap<String, String> = HashMap::new();
    
    let tournaments = get_paged_query(
    async |page| { make_request(&client, &GetTournaments::build_query(get_tournaments::Variables {page: Some(page)})).await },
    |res: &get_tournaments::ResponseData| -> Option<i64> { res.tournaments.as_ref()?.page_info.as_ref()?.total_pages },
    &|res: get_tournaments::ResponseData| { res.tournaments?.nodes }).await
    .expect("tournaments query");

    for tournament in tournaments.into_iter()
            .filter_map(|x| x)
            .filter(|t| t.name.as_ref().unwrap().contains("Trial")) {
        let id = tournament.id.as_ref().unwrap();
        // sql.execute("INSERT INTO Tournament VALUES (?1, ?2, ?3)", (id, tournament.name.as_ref().unwrap(), tournament.start_at.as_ref().unwrap()))?;
        for event in tournament.events.iter().flatten().filter_map(|x| x.as_ref()) {
            // sql.execute("INSERT INTO Event VALUES (?1, ?2, ?3)", (event.id.as_ref().unwrap(), event.name.as_ref().unwrap(), id))
            // .expect("Event insert");
        }
        sleep(Duration::from_millis(300)).await;
        let participants = get_paged_query(
            async |page| { make_request(&client, &GetParticipants::build_query(get_participants::Variables {id: tournament.id.clone(), page: Some(page), page_size: Some(60)})).await },
            |res: &get_participants::ResponseData| -> Option<i64> { res.tournament.as_ref()?.participants.as_ref()?.page_info.as_ref()?.total_pages },
            &|res: get_participants::ResponseData| { res.tournament?.participants?.nodes }).await
            .expect("participants query");
        
        for participant in participants.into_iter().filter_map(|x| x) {
            let pid = participant.player.as_ref().and_then(|p| p.id.clone()).expect("Missing id");
            if !players.contains(&pid) {
                // sql.execute("INSERT INTO Player VALUES (?1, ?2) ON CONFLICT DO NOTHING", (&pid, participant.player.as_ref().and_then(|p|p.gamer_tag.as_ref()).expect("Missing id")))
                // .expect("Player insert");
                players.insert(pid.clone());
            }
            for entrant in participant.entrants.into_iter().flatten().filter_map(|x| x) {
                entrants.insert(entrant.id.as_ref().expect("").clone(), pid.clone());
                // if entrant.standing.as_ref().and_then(|f|f.is_final).unwrap_or(false) {
                //     sql.execute("INSERT INTO Standing VALUES (?1, ?2, ?3)", 
                //         (&pid, 
                //             entrant.event.expect("").id.expect(""), 
                //             entrant.standing.unwrap().placement.unwrap()
                //         )).expect("Standing insert");
                // }
                sql.execute("UPDATE SetResult SET winnerID = ?1 WHERE winnerID = ?2", (&pid, entrant.id.as_ref()))?;
                sql.execute("UPDATE SetResult SET loserID = ?1 WHERE loserID = ?2", (&pid, entrant.id.as_ref()))?;
            }
        }
        continue;
        let arr : Vec<Option<String>> = tournament.events.as_ref().expect("").iter().map(|e| e.as_ref()?.id.clone()).collect();

        for next_ten in arr.chunks(1) {
            let events = get_paged_query(async |page| { make_request(&client, &GetSets::build_query(get_sets::Variables { id: Some(id.clone()), events: Some(next_ten.iter().cloned().collect()), page: Some(page), page_size: Some(60)})).await },
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
                            i if i > 0 => (Some(slot1), Some(slot2)),
                            i if i < 0 => (Some(slot2), Some(slot1)),
                            _ => (None, None) } {
                    if let (Some(winner_id), Some(loser_id)) = (winner.entrant.as_ref().and_then(|e| e.id.as_ref()), loser.entrant.as_ref().and_then(|e| e.id.as_ref())) {
                    if let (Some(winner_player_id), Some(loser_player_id)) = (entrants.get(winner_id), entrants.get(loser_id)) {
                    if let (Some(winner_score), Some(loser_score)) = (winner.standing.as_ref().and_then(|st|st.stats.as_ref()).and_then(|s|s.score.as_ref()).and_then(|sc|sc.value), loser.standing.as_ref().and_then(|st|st.stats.as_ref()).and_then(|s|s.score.as_ref()).and_then(|sc|sc.value)) {
                        if loser_score >= 0.0 { sql.execute("INSERT INTO SetResult VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)", (&set.id, &event.id, winner_player_id, loser_player_id, winner_score, loser_score, started_at.and_then(|s| Some(completed_at - s))))?; }
                    }}}}}}}}
                }
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