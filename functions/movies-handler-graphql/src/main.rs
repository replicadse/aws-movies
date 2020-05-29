#[macro_use]
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;

#[macro_use]
mod macros;
mod data;
mod option;
mod schema;
use crate::schema::create_schema;

use juniper::http::GraphQLRequest;
use lambda::error::HandlerError;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        LambdaError(::lambda::error::HandlerError);
        LoggerError(::log::SetLoggerError);
    }
}

pub fn execute() -> std::result::Result<(), HandlerError> {
    lambda!(handler);
    Ok(())
}

pub fn handler(
    request: GraphQLRequest,
    _: lambda::Context,
) -> std::result::Result<String, HandlerError> {
    info!("{:?}", request);
    let schema = create_schema();
    Ok(serde_json::to_string(&request.execute(&schema, &())).unwrap())
}

fn main() -> Result<()> {
    simple_logger::init_with_level(log::Level::Info)?;
    lambda!(handler);
    Ok(())
}
