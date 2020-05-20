#[macro_use]
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate log;

#[macro_use]
mod macros;
mod data;
mod schema;
use crate::schema::create_schema;

use std::error::Error;
use lambda::error::HandlerError;
use juniper::http::GraphQLRequest;

pub fn execute() -> Result<(), Box<dyn Error>> {
    lambda!(handler);
    Ok(())
}

pub fn handler(request: GraphQLRequest, _: lambda::Context) -> Result<String, HandlerError> {
    info!("{:?}", request);
    let schema = create_schema();
    Ok(serde_json::to_string(&request.execute(&schema, &())).unwrap())
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Info)?;
    lambda!(handler);
    Ok(())
}
