use reqwest::{Client, StatusCode};
use serde::{Serialize, Deserialize};
use std::{env, fs::File, io::Write};
use tokio;

// all variables in the following structs don't follow snake casing becuase they're used for the query
#[derive(Serialize)]
struct GraphQLRequest<'a> {
    query: &'a str,
    variables: GraphQLVariables,
}

#[derive(Serialize)]
struct GraphQLVariables {
    searchCriteria: TournamentQuery,
    videoGameID: i32,  
}

#[derive(Serialize)]
struct TournamentQuery {
    filter: TournamentFilter,
    sort: String,
    page: i32,
    perPage: i32,  
}

#[derive(Serialize)]
struct TournamentFilter {
    name: String,
    location: Location,
    afterDate: i64,  
}

#[derive(Serialize)]
struct Location {
    distanceFrom: String,  
    distance: String,
}

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

    // Define the variables for the query
    let variables = GraphQLVariables {
        searchCriteria: TournamentQuery {
            filter: TournamentFilter {
                name: "Tuesday Trials".to_string(),
                location: Location {
                    distanceFrom: "39.99386797577957, -83.00544391215689".to_string(),
                    distance: "5mi".to_string(),
                },
                afterDate: 1728561081,  // The timestamp you provided
            },
            sort: "startAt".to_string(),
            page: 1,
            perPage: 100,  
        },
        videoGameID: 33945,  // The video game ID is now directly set (camelCase)
    };

    // Set up the GraphQL request payload
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
        }
        StatusCode::OK => {
            // The response is a successful JSON response.
            let json = response.text().await.unwrap();
            //println!("Response: {:#?}", json);

            // Specify the file path where you want to save the JSON response
            let file_path = ".\\output\\response.json";

            // Write the JSON response to the file
            match write_to_file(file_path, &json) {
                Ok(_) => println!("Response saved to {}", file_path),
                Err(e) => println!("Failed to write to file: {}", e),
            }
        }
        _ => {
            println!("Reqwest Error");
        }
    }
}

fn write_to_file(file_path: &str, data: &str) -> std::io::Result<()> {
    let mut file = File::create(file_path)?;  // Creates the file at the specified path
    file.write_all(data.as_bytes())?;  // Writes the JSON data to the file
    Ok(())
}
