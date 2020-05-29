use crate::option::OptionMutExt;
use chrono::{DateTime, Datelike, Utc};
use juniper::GraphQLObject;
use rusoto_core::Region;
use rusoto_dynamodb::{
    AttributeValue, DynamoDb, DynamoDbClient, GetItemInput, PutItemInput, QueryInput,
};
use serde::{
    de::{Deserializer, MapAccess, Visitor},
    ser::{SerializeStruct, Serializer},
    Deserialize, Serialize,
};
use std::{collections::HashMap, fmt, str::FromStr};

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

#[derive(Debug)]
enum MovieKindItem {
    Meta {
        title: String,
        published_at: DateTime<Utc>,
        imdb_id: Option<String>,
    },
    Actor {
        characters: Vec<String>,
    },
}
#[derive(Debug)]
enum ActorKindItem {
    Meta {
        last_name: String,
        first_name: String,
    },
}

#[derive(Debug)]
enum DynamoTableItemKind {
    Movie { kind: MovieKindItem },
    Actor { kind: ActorKindItem },
}

#[derive(Debug)]
struct DynamoTableItem {
    pk: String,
    sk: String,
    kind: DynamoTableItemKind,
}

#[derive(Debug)]
enum DynamoTableRowKind {
    MovieMeta,
    MovieActor,
    ActorMeta,
}

impl DynamoTableItem {
    fn get_row_kind_by_keys(pk: &str, sk: &str) -> Result<DynamoTableRowKind> {
        if pk.starts_with("movie::") {
            if sk == "meta" {
                Ok(DynamoTableRowKind::MovieMeta)
            } else if sk.starts_with("actor::") {
                Ok(DynamoTableRowKind::MovieActor)
            } else {
                Err("unknown".into())
            }
        } else if pk.starts_with("actor::") {
            Ok(DynamoTableRowKind::ActorMeta)
        } else {
            Err("unknown".into())
        }
    }

    fn new_movie(movie: &Movie) -> Vec<Self> {
        let movie_item = DynamoTableItem {
            pk: format!(
                "movie::{}#{}",
                movie.meta.title,
                movie.meta.published_at.year()
            )
            .to_owned(),
            sk: "meta".to_owned(),
            kind: DynamoTableItemKind::Movie {
                kind: MovieKindItem::Meta {
                    title: movie.meta.title.clone(),
                    published_at: movie.meta.published_at,
                    imdb_id: movie.meta.imdb_id.clone(),
                },
            },
        };
        let mut movie_actor_items = movie
            .roles
            .iter()
            .map(|a| DynamoTableItem {
                pk: format!(
                    "movie::{}#{}",
                    movie.meta.title,
                    movie.meta.published_at.year()
                )
                .to_owned(),
                sk: format!("actor::{} {}", a.actor.last_name, a.actor.first_name),
                kind: DynamoTableItemKind::Movie {
                    kind: MovieKindItem::Actor {
                        characters: a.characters.iter().map(|c| c.name.clone()).collect(),
                    },
                },
            })
            .collect::<Vec<DynamoTableItem>>();
        let mut actor_items = movie
            .roles
            .iter()
            .map(|a| DynamoTableItem::new_actor(&a.actor))
            .collect::<Vec<DynamoTableItem>>();

        let mut items = vec![movie_item];
        items.append(&mut movie_actor_items);
        items.append(&mut actor_items);
        items
    }

    fn new_actor(actor: &Actor) -> Self {
        DynamoTableItem {
            pk: format!("actor::{} {}", actor.last_name, actor.first_name),
            sk: "meta".to_owned(),
            kind: DynamoTableItemKind::Actor {
                kind: ActorKindItem::Meta {
                    last_name: actor.last_name.clone(),
                    first_name: actor.first_name.clone(),
                },
            },
        }
    }
}

impl Serialize for DynamoTableItem {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.kind {
            DynamoTableItemKind::Movie { kind } => match kind {
                MovieKindItem::Meta {
                    title,
                    imdb_id,
                    published_at,
                } => {
                    let mut state = serializer.serialize_struct("", 6)?;
                    state.serialize_field("pk", &self.pk)?;
                    state.serialize_field("sk", &self.sk)?;
                    state.serialize_field("title", &title)?;
                    state.serialize_field("imdb_id", &imdb_id)?;
                    state.serialize_field("published_at", &published_at)?;
                    state.serialize_field("published_year", &published_at.year())?;
                    state.end()
                }
                MovieKindItem::Actor { characters } => {
                    let mut state = serializer.serialize_struct("", 3)?;
                    state.serialize_field("pk", &self.pk)?;
                    state.serialize_field("sk", &self.sk)?;
                    state.serialize_field("characters", &characters)?;
                    state.end()
                }
            },
            DynamoTableItemKind::Actor { kind } => match kind {
                ActorKindItem::Meta {
                    last_name,
                    first_name,
                } => {
                    let mut state = serializer.serialize_struct("", 5)?;
                    state.serialize_field("pk", &self.pk)?;
                    state.serialize_field("sk", &self.sk)?;
                    state.serialize_field("last_name", &last_name)?;
                    state.serialize_field("first_name", &first_name)?;
                    state.end()
                }
            },
        }
    }
}

impl<'de> Deserialize<'de> for DynamoTableItem {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ItemVisitor;
        impl<'de> Visitor<'de> for ItemVisitor {
            type Value = DynamoTableItem;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DynamoTableItem, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut data = HashMap::<String, serde_json::Value>::new();
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_ref() {
                        "pk" | "sk" | "title" | "imdb_id" | "last_name" | "first_name" => {
                            data.insert(
                                key.to_owned(),
                                serde_json::to_value(map.next_value::<String>()?).unwrap(),
                            );
                        }
                        "characters" => {
                            data.insert(
                                key.to_owned(),
                                serde_json::to_value(map.next_value::<Vec<String>>()?).unwrap(),
                            );
                        }
                        "published_at" => {
                            data.insert(
                                key.to_owned(),
                                serde_json::to_value(map.next_value::<DateTime<Utc>>()?).unwrap(),
                            );
                        }
                        "published_year" => {
                            map.next_value::<i32>()?;
                        }
                        _ => return Err(serde::de::Error::unknown_field(key.as_ref(), &[])),
                    }
                }
                let pk = data.get("pk").unwrap().as_str().unwrap().to_owned();
                let sk = data.get("sk").unwrap().as_str().unwrap().to_owned();
                let kind = DynamoTableItem::get_row_kind_by_keys(&pk, &sk).unwrap();
                match kind {
                    DynamoTableRowKind::MovieMeta => Ok(DynamoTableItem {
                        pk,
                        sk,
                        kind: DynamoTableItemKind::Movie {
                            kind: MovieKindItem::Meta {
                                title: data.get("title").unwrap().as_str().unwrap().to_owned(),
                                imdb_id: data
                                    .get("imdb_id")
                                    .unwrap()
                                    .as_str()
                                    .mutate(|x| Some(x.to_owned())),
                                published_at: serde_json::from_value(
                                    data.get("published_at").unwrap().clone(),
                                )
                                .unwrap(),
                            },
                        },
                    }),
                    DynamoTableRowKind::MovieActor => Ok(DynamoTableItem {
                        pk,
                        sk,
                        kind: DynamoTableItemKind::Movie {
                            kind: MovieKindItem::Actor {
                                characters: serde_json::from_value(
                                    data.get("characters").unwrap().clone(),
                                )
                                .unwrap(),
                            },
                        },
                    }),
                    DynamoTableRowKind::ActorMeta => Ok(DynamoTableItem {
                        pk,
                        sk,
                        kind: DynamoTableItemKind::Actor {
                            kind: ActorKindItem::Meta {
                                last_name: data
                                    .get("last_name")
                                    .unwrap()
                                    .as_str()
                                    .unwrap()
                                    .to_owned(),
                                first_name: data
                                    .get("first_name")
                                    .unwrap()
                                    .as_str()
                                    .unwrap()
                                    .to_owned(),
                            },
                        },
                    }),
                }
            }
        }
        deserializer.deserialize_struct("", &[], ItemVisitor {})
    }
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
                s: Some(format!("movie::{}#{}", title, published_at.year()).to_owned()),
                ..Default::default()
            },
            "sk".to_owned() => AttributeValue {
                s: Some("meta".to_owned()),
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
                s: Some(format!("actor::{}", name).to_owned()),
                ..Default::default()
            },
            "sk".to_owned() => AttributeValue {
                s: Some("meta".to_owned()),
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
                s: Some(format!("movie::{}#{}", title, published_at.year()).to_owned()),
                ..Default::default()
            },
            ":sk".to_owned() => AttributeValue {
                s: Some("actor::".to_owned()),
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
                                actor: read_actor_metadata(role.sk.trim_start_matches("actor::"))
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

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use std::str::FromStr;

    #[test]
    fn test_dynamotableitem_serialization() {
        let movie = crate::data::Movie {
            meta: crate::data::MovieMetadata {
                title: "The Irishman".to_owned(),
                imdb_id: Some("tt1302006".to_owned()),
                published_at: DateTime::<Utc>::from_str("2019-09-27T00:00:00Z").unwrap(),
            },
            roles: vec![
                crate::data::Role {
                    actor: crate::data::Actor {
                        last_name: "Deniro".to_owned(),
                        first_name: "Robert".to_owned(),
                    },
                    characters: vec![crate::data::Character {
                        name: "Frank Sheeran".to_owned(),
                    }],
                },
                crate::data::Role {
                    actor: crate::data::Actor {
                        last_name: "Pacino".to_owned(),
                        first_name: "Al".to_owned(),
                    },
                    characters: vec![crate::data::Character {
                        name: "Jimmy Hoffa".to_owned(),
                    }],
                },
            ],
        };

        let table_items = crate::data::DynamoTableItem::new_movie(&movie)
            .iter()
            .map(|x| serde_json::to_string(&x).unwrap())
            .collect::<Vec<String>>();
        let expected_table_items = vec![
            r#"{"pk":"movie::The Irishman#2019","sk":"meta","title":"The Irishman","imdb_id":"tt1302006","published_at":"2019-09-27T00:00:00Z","published_year":2019}"#,
            r#"{"pk":"movie::The Irishman#2019","sk":"actor::Deniro Robert","characters":["Frank Sheeran"]}"#,
            r#"{"pk":"movie::The Irishman#2019","sk":"actor::Pacino Al","characters":["Jimmy Hoffa"]}"#,
            r#"{"pk":"actor::Deniro Robert","sk":"meta","last_name":"Deniro","first_name":"Robert"}"#,
            r#"{"pk":"actor::Pacino Al","sk":"meta","last_name":"Pacino","first_name":"Al"}"#,
        ];
        assert_eq!(expected_table_items, table_items);
    }

    #[test]
    fn test_deserialize_movie_meta() {
        let data = r#"{"pk":"movie::The Irishman#2019","sk":"meta","title":"The Irishman","imdb_id":"tt1302006","published_at":"2019-09-27T00:00:00Z","published_year":2019}"#;
        let item: crate::data::DynamoTableItem = serde_json::from_str(data).unwrap();
        println!("{:?}", item);
    }

    #[test]
    fn test_deserialize_movie_actor() {
        let data = r#"{"pk":"movie::The Irishman#2019","sk":"actor::Deniro Robert","characters":["Frank Sheeran"]}"#;
        let item: crate::data::DynamoTableItem = serde_json::from_str(data).unwrap();
        println!("{:?}", item);
    }

    #[test]
    fn test_deserialize_actor_meta() {
        let data = r#"{"pk":"actor::Deniro Robert","sk":"meta","last_name":"Deniro","first_name":"Robert"}"#;
        let item: crate::data::DynamoTableItem = serde_json::from_str(data).unwrap();
        println!("{:?}", item);
    }
}
