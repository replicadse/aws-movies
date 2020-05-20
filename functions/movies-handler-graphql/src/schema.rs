use juniper::{FieldResult, FieldError, RootNode, GraphQLInputObject};
use chrono::{DateTime, Utc};

pub struct QueryRoot;
pub struct MutationRoot;
pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}

#[derive(Debug, Clone, GraphQLInputObject)]
struct PostMovieRequest {
    title: String,
    watched: Option<DateTime<Utc>>,
    actors: Option<Vec<String>>,
}

#[juniper::object]
impl QueryRoot {
    #[graphql(name = "get_movie")]
    fn get_human(id: String) -> FieldResult<crate::data::Movie> {
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        match runtime.block_on(crate::data::read_item(&id)) {
            Ok(m) => Ok(m),
            Err(e) => Err(FieldError::from(e))
        }
    }

    #[graphql(name = "list_movies")]
    fn get_human() -> FieldResult<Vec<String>> {
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        match runtime.block_on(crate::data::scan_item_ids()) {
            Ok(m) => Ok(m),
            Err(e) => Err(FieldError::from(e))
        }
    }
}

#[juniper::object]
impl MutationRoot {
    #[graphql(name = "post_movie")]
    fn post_movie(request: PostMovieRequest) -> FieldResult<String> {
        let movie = crate::data::Movie {
            id: uuid::Uuid::new_v4().to_string(),
            title: request.title,
            watched: request.watched,
            actors: request.actors.unwrap_or(Vec::new()),
        };
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        match runtime.block_on(crate::data::store_item(movie)) {
            Ok(m) => Ok(m),
            Err(e) => Err(FieldError::from(e))
        }
    }

    #[graphql(name = "delete_movie")]
    fn delete_movie(id: String) -> FieldResult<bool> {
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        match runtime.block_on(crate::data::delete_item(&id)) {
            Ok(_) => Ok(true),
            Err(e) => Err(FieldError::from(e))
        }
    }
}
