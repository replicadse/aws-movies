use std::io::Error;
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use juniper::GraphQLObject;
use chrono::{DateTime, Utc};
use rusoto_core::Region;
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, AttributeValue, PutItemInput, GetItemInput, ScanInput, DeleteItemInput};

#[derive(Debug, Clone, Serialize, Deserialize, GraphQLObject)]
#[serde(rename_all = "snake_case")]
pub struct Movie {
    pub id: String,
    pub title: String,
    pub watched: Option<DateTime<Utc>>,
    pub actors: Vec<String>,
}

async fn get_region() -> Region {
    Region::from_str(std::env::var("AWS_REGION").unwrap().as_ref()).unwrap()
}

async fn get_table_name() -> String {
    std::env::var("TABLE_NAME").unwrap()
}

pub async fn read_item(id: &str) -> Result<Movie, Error> {
    let client = DynamoDbClient::new(get_region().await);
    let input = GetItemInput {
        table_name: get_table_name().await,
        key: map!{
            "id".to_owned() => AttributeValue {
                s: Some(id.to_owned()),
                ..Default::default()
            }
        },
        ..Default::default()
    };
    match client.get_item(input).await {
        Ok(output) => {
            let item = output.item.unwrap();
            let movie: Movie = serde_dynamodb::from_hashmap(item).unwrap();
            Ok(movie)
        },
        Err(e) => Err(Error::new(std::io::ErrorKind::Other, e))
    }
}

pub async fn store_item(movie: Movie) -> Result<String, Error> {
    let client = DynamoDbClient::new(get_region().await);
    let input = PutItemInput {
        table_name: get_table_name().await,
        item: serde_dynamodb::to_hashmap(&movie).unwrap(),
        ..Default::default()
    };
    match client.put_item(input).await {
        Ok(_) => Ok(movie.id),
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
    }
}

pub async fn delete_item(id: &str) -> Result<(), Error> {
    if id.is_empty() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "empty id"))
    }
    let client = DynamoDbClient::new(get_region().await);
    let input = DeleteItemInput {
        table_name: get_table_name().await,
        key: map!{"id".to_owned() => AttributeValue {
            s: Some(id.to_owned()),
            ..Default::default()
        }},
        ..Default::default()
    };
    match client.delete_item(input).await {
        Ok(_) => Ok(()),
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
    }
}

pub async fn scan_item_ids() -> Result<Vec<String>, Error> {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct IDContainer {
        id: String,
    }

    let client = DynamoDbClient::new(get_region().await);
    let input = ScanInput {
        table_name: get_table_name().await,
        attributes_to_get: Some(vec!["id".to_owned()]),
        ..Default::default()
    };
    match client.scan(input).await {
        Ok(out) => {
            Ok(out.items.unwrap().iter().map(|e| {
                let id: IDContainer = serde_dynamodb::from_hashmap(e.clone()).unwrap();
                id.id
            }).collect())
        },
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
    }
}
