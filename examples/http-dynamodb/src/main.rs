use lambda_http::{run, service_fn, Error, IntoResponse, Request, RequestExt, Response};
use serde_json::Value;
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::{Client, Error as OtherError};
use aws_sdk_config::Region;
use tracing::info;

pub struct Item {
    pub p_type: String,
    pub age: String,
    pub username: String,
    pub first: String,
    pub last: String,
}

/// This is the main body for the function.
/// Write your code inside it.
/// You can see more examples in Runtime's repository:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
async fn function_handler(event: Request) -> Result<impl IntoResponse, Error> {
    // Extract some useful information from the request

    let body = event.body();
    let s = std::str::from_utf8(&body).expect("invalid utf-8 sequence");
    // Parse the string of data into serde_json::Value.

    info!("JSON Payload sent: {}", s);
    let parsed: Value = serde_json::from_str(s).unwrap();
    
    let item = Item {
        p_type : parsed["ptype"].as_str().unwrap().to_string(),
        age : parsed["age"].as_str().unwrap().to_string(), 
        username : parsed["username"].as_str().unwrap().to_string(),
        first : parsed["firstname"].as_str().unwrap().to_string(),
        last : parsed["lastname"].as_str().unwrap().to_string() 
    };

    let region =  Region::new("ca-central-1");
    let shared_config = aws_config::from_env().region(region).load().await;
    let client = Client::new(&shared_config);
  
    add_item(&client, item, &"lambda_dyno_example".to_string()).await?;
    
    Ok(match event.query_string_parameters().first("first_name") {
        Some(first_name) => format!("Hello, {}!", first_name).into_response().await,
        _ => Response::builder()
            .status(400)
            .body("Empty first name".into())
            .expect("failed to render response"),
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}

// Add an item to a table.
// snippet-start:[dynamodb.rust.add-item]
pub async fn add_item(client: &Client, item: Item, table: &String) -> Result<(), OtherError> {
    let user_av = AttributeValue::S(item.username);
    let type_av = AttributeValue::S(item.p_type);
    let age_av = AttributeValue::S(item.age);
    let first_av = AttributeValue::S(item.first);
    let last_av = AttributeValue::S(item.last);

    let request = client
        .put_item()
        .table_name(table)
        .item("username", user_av)
        .item("account_type", type_av)
        .item("age", age_av)
        .item("first_name", first_av)
        .item("last_name", last_av);

    println!("Executing request [{request:?}] to add item...\n\n\n\n");

    let _resp = request.send().await?;

    Ok(())
}