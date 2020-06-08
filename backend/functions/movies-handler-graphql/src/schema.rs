use chrono::{prelude::*, DateTime, Utc};
use juniper::{FieldResult, GraphQLInputObject, RootNode};
use serde::{Deserialize, Serialize};

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        Data(crate::data::Error, crate::data::ErrorKind);
    }
}

pub struct QueryRoot;
pub struct MutationRoot;
pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLInputObject)]
#[serde(rename_all = "snake_case")]
struct PutMovieRequestRole {
    #[graphql(name = "actor_last_name")]
    actor_last_name: String,
    #[graphql(name = "actor_first_name")]
    actor_first_name: String,
    #[graphql(name = "character_names")]
    character_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLInputObject)]
#[serde(rename_all = "snake_case")]
struct PutMovieRequest {
    #[graphql(name = "title")]
    title: String,
    #[graphql(name = "imdb_id")]
    imdb_id: Option<String>,
    #[graphql(name = "published_at")]
    published_at: DateTime<Utc>,
    #[graphql(name = "roles")]
    roles: Vec<PutMovieRequestRole>,
}

#[juniper::object]
impl QueryRoot {
    #[graphql(name = "get_movie")]
    fn get_movie(title: String, published: i32) -> FieldResult<crate::data::Movie> {
        let published_dt = DateTime::from_utc(
            NaiveDate::from_ymd(published, 1, 1).and_time(NaiveTime::from_hms(0, 0, 0)),
            Utc,
        );
        let mut runtime = tokio::runtime::Runtime::new()?;
        match runtime.block_on(crate::storage::read_movie(&title, &published_dt)) {
            Ok(m) => Ok(m),
            Err(e) => Err(e.to_string().into()),
        }
    }
}

#[juniper::object]
impl MutationRoot {
    #[graphql(name = "put_movie")]
    fn put_movie(request: PutMovieRequest) -> FieldResult<bool> {
        let movie = crate::data::Movie {
            meta: crate::data::MovieMetadata {
                title: request.title,
                imdb_id: request.imdb_id,
                published_at: request.published_at,
            },
            roles: request
                .roles
                .iter()
                .map(|r| crate::data::Role {
                    actor: crate::data::Actor {
                        first_name: r.actor_first_name.clone(),
                        last_name: r.actor_last_name.clone(),
                    },
                    characters: r
                        .character_names
                        .iter()
                        .map(|x| crate::data::Character { name: x.to_owned() })
                        .collect(),
                })
                .collect(),
        };
        let mut runtime = tokio::runtime::Runtime::new()?;
        match runtime.block_on(crate::storage::store_movie(movie)) {
            Ok(_) => Ok(true),
            Err(e) => Err(e.to_string().into()),
        }
    }
}
