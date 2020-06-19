use chrono::{DateTime, Utc};
use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        SerdeDynamo(::serde_dynamodb::Error);
        Var(::std::env::VarError);
        ParseRegionError(::rusoto_core::region::ParseRegionError);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLObject)]
#[serde(rename_all = "snake_case")]
pub struct Movie {
    #[graphql(name = "meta")]
    pub meta: MovieMetadata,
    #[graphql(name = "roles")]
    pub roles: Vec<Role>,
}

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLObject)]
#[serde(rename_all = "snake_case")]
pub struct MovieMetadata {
    #[graphql(name = "title")]
    pub title: String,
    #[graphql(name = "imdb_id")]
    pub imdb_id: Option<String>,
    #[graphql(name = "published_at")]
    pub published_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLObject)]
#[serde(rename_all = "snake_case")]
pub struct Role {
    #[graphql(name = "actor")]
    pub actor: Actor,
    #[graphql(name = "characters")]
    pub characters: Vec<Character>,
}

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLObject)]
#[serde(rename_all = "snake_case")]
pub struct Character {
    #[graphql(name = "name")]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLObject)]
#[serde(rename_all = "snake_case")]
pub struct Actor {
    #[graphql(name = "last_name")]
    pub last_name: String,
    #[graphql(name = "first_name")]
    pub first_name: String,
}
