use crate::dynamo::*;
use chrono::{DateTime, Datelike, Utc};
use juniper::GraphQLObject;
use rusoto_core::Region;
use rusoto_dynamodb::{
    AttributeValue, DynamoDb, DynamoDbClient, GetItemInput, PutItemInput, QueryInput,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

async fn get_region() -> Result<Region> {
    match Region::from_str(std::env::var("AWS_REGION")?.as_ref()) {
        Ok(r) => Ok(r),
        Err(e) => Err(e.to_string().into()),
    }
}

async fn get_table_name() -> Result<String> {
    match std::env::var("TABLE_NAME") {
        Ok(r) => Ok(r),
        Err(e) => Err(e.to_string().into()),
    }
}

pub async fn read_movie(title: &str, published_at: &DateTime<Utc>) -> Result<Movie> {
    Ok(Movie {
        meta: read_movie_metadata(title, published_at).await?,
        roles: read_movie_roles(title, published_at).await?,
    })
}

pub async fn read_movie_metadata(
    title: &str,
    published_at: &DateTime<Utc>,
) -> Result<MovieMetadata> {
    let client = DynamoDbClient::new(get_region().await?);
    let input = GetItemInput {
        table_name: get_table_name().await?,
        key: map! {
            "pk".to_owned() => AttributeValue {
                s: Some(format!("{}{}#{}", DynamoTableRowKind::MovieMeta.get_prefixes().0, title, published_at.year()).to_owned()),
                ..Default::default()
            },
            "sk".to_owned() => AttributeValue {
                s: Some(DynamoTableRowKind::MovieMeta.get_prefixes().1),
                ..Default::default()
            }
        },
        ..Default::default()
    };

    match client.get_item(input).await {
        Ok(output) => {
            let item = output
                .item
                .ok_or_else(|| Into::<Error>::into("none error"))?;
            let meta: DynamoTableItem = serde_dynamodb::from_hashmap(item)?;
            match meta.kind {
                DynamoTableItemKind::Movie { kind, .. } => match kind {
                    MovieKindItem::Meta {
                        title,
                        imdb_id,
                        published_at,
                    } => Ok(MovieMetadata {
                        title,
                        imdb_id,
                        published_at,
                    }),
                    _ => Err("nope".into()),
                },
                _ => Err("nope".into()),
            }
        }
        Err(e) => Err(e.to_string().into()),
    }
}

pub async fn read_actor_metadata(name: &str) -> Result<Actor> {
    let client = DynamoDbClient::new(get_region().await?);
    let input = GetItemInput {
        table_name: get_table_name().await?,
        key: map! {
            "pk".to_owned() => AttributeValue {
                s: Some(format!("{}{}", DynamoTableRowKind::ActorMeta.get_prefixes().0, name).to_owned()),
                ..Default::default()
            },
            "sk".to_owned() => AttributeValue {
                s: Some(DynamoTableRowKind::ActorMeta.get_prefixes().1),
                ..Default::default()
            }
        },
        ..Default::default()
    };

    match client.get_item(input).await {
        Ok(output) => {
            let item = output
                .item
                .ok_or_else(|| Into::<Error>::into("none error"))?;
            let meta: DynamoTableItem = serde_dynamodb::from_hashmap(item)?;
            match meta.kind {
                DynamoTableItemKind::Actor { kind, .. } => match kind {
                    ActorKindItem::Meta {
                        last_name,
                        first_name,
                    } => Ok(Actor {
                        last_name,
                        first_name,
                    }),
                },
                _ => Err("nope".into()),
            }
        }
        Err(e) => Err(e.to_string().into()),
    }
}

pub async fn read_movie_roles(title: &str, published_at: &DateTime<Utc>) -> Result<Vec<Role>> {
    let client = DynamoDbClient::new(get_region().await?);
    let input = QueryInput {
        table_name: get_table_name().await?,
        expression_attribute_values: Some(map! {
            ":pk".to_owned() => AttributeValue {
                s: Some(format!("{}{}#{}", DynamoTableRowKind::MovieActor.get_prefixes().0, title, published_at.year()).to_owned()),
                ..Default::default()
            },
            ":sk".to_owned() => AttributeValue {
                s: Some(DynamoTableRowKind::MovieActor.get_prefixes().1),
                ..Default::default()
            }
        }),
        key_condition_expression: Some("pk = :pk AND begins_with(sk, :sk)".to_owned()),
        ..Default::default()
    };
    let mut result = vec![];
    match client.query(input).await {
        Ok(o) => match o.items {
            Some(items) => {
                for item in items {
                    let role: DynamoTableItem = serde_dynamodb::from_hashmap(item)?;
                    match role.kind {
                        DynamoTableItemKind::Movie { kind } => match kind {
                            MovieKindItem::Actor { characters } => result.push(Role {
                                actor: read_actor_metadata(role.sk.trim_start_matches(
                                    &DynamoTableRowKind::MovieActor.get_prefixes().1,
                                ))
                                .await?,
                                characters: characters
                                    .iter()
                                    .map(|x| Character { name: x.clone() })
                                    .collect(),
                            }),
                            _ => {
                                return Err("nope".into());
                            }
                        },
                        _ => {
                            return Err("nope".into());
                        }
                    }
                }
            }
            None => {}
        },
        Err(e) => return Err(e.to_string().into()),
    };
    Ok(result)
}

pub async fn store_movie(movie: Movie) -> Result<()> {
    let client = DynamoDbClient::new(get_region().await?);
    let items = DynamoTableItem::new_movie(&movie);
    for item in items {
        let input = PutItemInput {
            table_name: get_table_name().await?,
            item: serde_dynamodb::to_hashmap(&item)?,
            ..Default::default()
        };
        client.put_item(input).await.unwrap();
    }
    Ok(())
}
