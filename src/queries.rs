use get_tournament::{GetTournamentTournament, GetTournamentTournamentEvents};
use get_tournaments::{GetTournamentsTournamentsNodes, GetTournamentsTournamentsNodesEvents};
use graphql_client::GraphQLQuery;

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

#[derive(GraphQLQuery)]
#[graphql(schema_path = "schema.graphql", query_path = "src/queries/GetTournament.graphql", variables_derives = "Debug")]
pub struct GetTournament;

#[derive(GraphQLQuery)]
#[graphql(schema_path = "schema.graphql", query_path = "src/queries/GetPlayer.graphql", variables_derives = "Debug")]
pub struct GetPlayer;

pub struct TournamentEvent {
    pub name: String,
    pub id: String,
}
impl TryFrom<GetTournamentsTournamentsNodesEvents> for TournamentEvent {
    type Error = &'static str;
    fn try_from(value: GetTournamentsTournamentsNodesEvents) -> Result<Self, Self::Error> {
        Ok(TournamentEvent { name: value.name.ok_or("Missing name")?, id: value.id.ok_or("Missing ID")? })
    }
}
impl TryFrom<GetTournamentTournamentEvents> for TournamentEvent {
    type Error = &'static str;
    fn try_from(value: GetTournamentTournamentEvents) -> Result<Self, Self::Error> {
        Ok(TournamentEvent { name: value.name.ok_or("Missing name")?, id: value.id.ok_or("Missing ID")? })
    }
}
pub struct Tournament {
    pub id: String,
    pub name: String,
    pub start_at: Option<i64>,
    pub events: Option<Vec<TournamentEvent>>
}
impl TryFrom<GetTournamentsTournamentsNodes> for Tournament {
    type Error = &'static str;
    
    fn try_from(value: GetTournamentsTournamentsNodes) -> Result<Self, Self::Error> {
        Ok(Tournament {id: value.id.ok_or("Missing ID")?,
            name: value.name.ok_or("Missing name")?,
            start_at: value.start_at,
            events: match value.events {
                None => None,
                Some(events) => Some(events.into_iter().map(|e|TournamentEvent::try_from(e.ok_or("Missing event")?)).collect::<Result<Vec<_>, _>>()?)
            }})
    }
}
impl TryFrom<GetTournamentTournament> for Tournament {
    type Error = &'static str;

    fn try_from(value: GetTournamentTournament) -> Result<Self, Self::Error> {
        Ok(Tournament {id: value.id.ok_or("Missing ID")?,
            name: value.name.ok_or("Missing name")?,
            start_at: value.start_at,
            events: match value.events {
                None => None,
                Some(events) => Some(events.into_iter().map(|e|TournamentEvent::try_from(e.ok_or("Missing event")?)).collect::<Result<Vec<_>, _>>()?)
            }})
    }
}