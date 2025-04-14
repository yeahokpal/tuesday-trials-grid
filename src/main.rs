use chrono::{Duration, Utc};
use reqwest::{Client, StatusCode};
use serde::Serialize;
use serde_json::Value;
use std::{collections::HashSet, env, fs::{self, File}, io::Write, process};
use tokio;

/* #region Query Structs */

// all variables in the following structs don't follow snake casing becuase they're used for the query
#[derive(Serialize)]
struct GraphQLRequest<'a> {
    query: &'a str,
    variables: GraphQLVariables,
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct GraphQLVariables {
    searchCriteria: TournamentQuery,
    videoGameID: i32,  
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct TournamentQuery {
    filter: TournamentFilter,
    sort: String,
    page: i32,
    perPage: i32,  
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct TournamentFilter {
    name: String,
    location: Location,
    afterDate: i64,  
    past: bool,
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct Location {
    distanceFrom: String,  
    distance: String,
}

/* #endregion */

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let api_token = env::var("STARTGG_API_TOKEN").expect("STARTGG_API_TOKEN must be set.");

    // Define the GraphQL query string
    let query = r#"
    query getTrialsEntrants($searchCriteria: TournamentQuery!, $videoGameID: ID) {
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
    }
    "#;

    let now = Utc::now();
    let six_months_ago = now - Duration::days(30 * 6); // Approx 6 months ago
    let timestamp = six_months_ago.timestamp();

    // Define the variables for the query
    let variables = GraphQLVariables {
        searchCriteria: TournamentQuery {
            filter: TournamentFilter {
                name: "Tuesday Trials".to_string(),
                past: true,
                location: Location {
                    distanceFrom: "39.99386797577957, -83.00544391215689".to_string(),
                    distance: "5mi".to_string(),
                },
                afterDate: timestamp,  // Unix timestamp for 6 months from current time
            },
            sort: "startAt".to_string(),
            page: 1,
            perPage: 100,  
        },
        videoGameID: 33945, // Start.gg ID for Guilty Gear Strive
    };

    // Set up the GraphQL request
    let body = GraphQLRequest {
        query,
        variables,
    };

    // Create a Reqwest client
    let client = Client::new();

    // Send the POST request
    let response = client
        .post("https://api.start.gg/gql/alpha")
        .header("Authorization", api_token)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)  // Send the payload as JSON
        .send()
        .await
        .unwrap();

    // Check the response status
    match response.status() {
        StatusCode::NOT_FOUND => {
            println!("Not Found");
            process::exit(0);
        }
        StatusCode::OK => {
            // The response is a successful JSON response.
            let json = response.text().await.unwrap();
            //println!("Response: {:#?}", json);

            // Specify the file path where you want to save the JSON response
            let file_path = ".\\output\\trials_names.json";

            // Write the JSON response to the file
            match write_to_file(file_path, &json) {
                Ok(_) => println!("Response saved to {}", file_path),
                Err(e) => {
                    println!("Failed to write to file: {}", e);
                    process::exit(0);
                }
            }
        }
        _ => {
            println!("Reqwest Error");
            process::exit(0);
        }
    }

    let _ = get_entrant_ids();
}

fn write_to_file(file_path: &str, data: &str) -> std::io::Result<()> {
    let mut file = File::create(file_path)?;  // Creates the file at the specified path
    file.write_all(data.as_bytes())?;  // Writes the JSON data to the file
    Ok(())
}

fn get_entrant_ids() -> std::io::Result<()> {
    let names_json = fs::read_to_string(".\\output\\trials_names.json")?;
    if names_json.contains("\"id\"") {
        println!("Found these IDs: ");
    } else {
        println!("No \"id\" found.");
    }

    let json: Value = serde_json::from_str(&names_json)?;
    let mut id_set = HashSet::new();
    extract_ids(&json, &mut id_set);

    println!("Unique IDs: {:?}", id_set);

    Ok(())

}
fn extract_ids(value: &Value, id_set: &mut HashSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                if key == "id" {
                    if let Some(id_str) = val.as_str() {
                        id_set.insert(id_str.to_string());
                    } else {
                        // Try converting to string anyway (e.g., for numbers)
                        id_set.insert(val.to_string());
                    }
                } else {
                    extract_ids(val, id_set);
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                extract_ids(item, id_set);
            }
        }
        _ => {}
    }
}
