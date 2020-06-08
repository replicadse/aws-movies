use crate::data::{Actor, Movie};
use crate::option::OptionMutExt;
use chrono::{DateTime, Datelike, Utc};
use serde::{
    de::{Deserializer, MapAccess, Visitor},
    ser::{SerializeStruct, Serializer},
    Deserialize, Serialize,
};
use std::{collections::HashMap, fmt};

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

#[derive(Debug)]
pub enum MovieKindItem {
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
pub enum ActorKindItem {
    Meta {
        last_name: String,
        first_name: String,
    },
}

#[derive(Debug)]
pub enum DynamoTableItemKind {
    Movie { kind: MovieKindItem },
    Actor { kind: ActorKindItem },
}

#[derive(Debug)]
pub struct DynamoTableItem {
    pub pk: String,
    pub sk: String,
    pub kind: DynamoTableItemKind,
}

#[derive(Debug)]
pub enum DynamoTableRowKind {
    MovieMeta,
    MovieActor,
    ActorMeta,
}

impl DynamoTableRowKind {
    pub fn get_prefixes(&self) -> (String, String) {
        match self {
            Self::MovieMeta => (String::from("movie::"), String::from("meta")),
            Self::MovieActor => (String::from("movie::"), String::from("actor::")),
            Self::ActorMeta => (String::from("actor::"), String::from("meta")),
        }
    }

    fn get_row_kind_by_keys(pk: &str, sk: &str) -> Result<Self> {
        let movie_pk = &Self::MovieMeta {}.get_prefixes().0;
        let movie_meta_sk = &Self::MovieMeta {}.get_prefixes().1;
        let movie_actor_sk = &Self::MovieActor {}.get_prefixes().1;
        let actor_pk = &Self::ActorMeta {}.get_prefixes().0;
        let actor_meta_sk = &Self::ActorMeta {}.get_prefixes().1;

        if pk.starts_with(movie_pk) {
            if sk == movie_meta_sk {
                Ok(DynamoTableRowKind::MovieMeta)
            } else if sk.starts_with(movie_actor_sk) {
                Ok(DynamoTableRowKind::MovieActor)
            } else {
                Err("unknown".into())
            }
        } else if pk.starts_with(actor_pk) {
            if sk.starts_with(actor_meta_sk) {
                Ok(DynamoTableRowKind::ActorMeta)
            } else {
                Err("unknown".into())
            }
        } else {
            Err("unknown".into())
        }
    }
}

impl DynamoTableItem {
    pub fn new_movie(movie: &Movie) -> Vec<Self> {
        let movie_item = DynamoTableItem {
            pk: format!(
                "{}{}#{}",
                DynamoTableRowKind::MovieMeta.get_prefixes().0,
                movie.meta.title,
                movie.meta.published_at.year()
            )
            .to_owned(),
            sk: DynamoTableRowKind::MovieMeta.get_prefixes().1,
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
                    "{}{}#{}",
                    DynamoTableRowKind::MovieActor.get_prefixes().0,
                    movie.meta.title,
                    movie.meta.published_at.year()
                )
                .to_owned(),
                sk: format!(
                    "{}{} {}",
                    DynamoTableRowKind::MovieActor.get_prefixes().1,
                    a.actor.last_name,
                    a.actor.first_name
                ),
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

    pub fn new_actor(actor: &Actor) -> Self {
        DynamoTableItem {
            pk: format!(
                "{}{} {}",
                DynamoTableRowKind::ActorMeta.get_prefixes().0,
                actor.last_name,
                actor.first_name
            ),
            sk: DynamoTableRowKind::ActorMeta.get_prefixes().1,
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
                let kind = DynamoTableRowKind::get_row_kind_by_keys(&pk, &sk).unwrap();
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

#[cfg(test)]
mod tests {
    use crate::dynamo::DynamoTableItem;
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

        let table_items = DynamoTableItem::new_movie(&movie)
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
        let item: DynamoTableItem = serde_json::from_str(data).unwrap();
        println!("{:?}", item);
    }

    #[test]
    fn test_deserialize_movie_actor() {
        let data = r#"{"pk":"movie::The Irishman#2019","sk":"actor::Deniro Robert","characters":["Frank Sheeran"]}"#;
        let item: DynamoTableItem = serde_json::from_str(data).unwrap();
        println!("{:?}", item);
    }

    #[test]
    fn test_deserialize_actor_meta() {
        let data = r#"{"pk":"actor::Deniro Robert","sk":"meta","last_name":"Deniro","first_name":"Robert"}"#;
        let item: DynamoTableItem = serde_json::from_str(data).unwrap();
        println!("{:?}", item);
    }
}
