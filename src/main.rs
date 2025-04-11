use dotenv::dotenv;
use reqwest::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE}, StatusCode
};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let api_token = std::env::var("STARTGG_API_TOKEN").expect("STARTGG_API_TOKEN must be set.");

    //let get_trials_entrants = fs::read_to_string("\\queries\\getTrialsEntrants.gql");

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.start.gg/gql/alpha")
        .header(AUTHORIZATION, api_token)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .body(
            "query getTrialsEntrants($searchCriteria: TournamentQuery!, $videoGameID: ID) {
    tournaments(query: $searchCriteria) {
        pageInfo {
            page
            perPage
            totalPages
        }
        nodes {
            name
            url
            events(filter: { videogameId: [$videoGameID] }) {
                name
                entrants {
                    nodes {
                        name
                        id
                    }
                }
            }
        }
    }
}",
        )
        .send()
        .await.unwrap();
    
    match response.status() {
        StatusCode::NOT_FOUND => {
            println!("Not Found");
        }
        StatusCode::OK => {
            println!("OK!");
            let json = response.json::<HashMap<String, String>>().await;
            println!("{:#?}", json);
        }
        _ => {
            println!("Reqwest Error");
        }
    }
    //println!("{:#}", body);
}
