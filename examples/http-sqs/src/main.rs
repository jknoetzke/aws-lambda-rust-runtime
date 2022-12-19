use aws_sdk_sqs::Client;
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use tracing::info;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// Which queue to use. If not provided, uses the first queue found.
    #[structopt(short, long)]
    queue: Option<String>,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Item {
    pub p_type: String,
    pub age: String,
    pub username: String,
    pub first: String,
    pub last: String,
}

#[derive(Debug)]
struct SQSMessage {
    body: String,
    group: String,
}

async fn send(client: &Client, queue_url: &String, message: &SQSMessage) -> Result<(), Error> {
    println!("Sending message to queue with URL: {}", queue_url);

    let rsp = client
        .send_message()
        .queue_url(queue_url)
        .message_body(&message.body)
        .message_group_id(&message.group)
        // If the queue is FIFO, you need to set .message_deduplication_id
        // or configure the queue for ContentBasedDeduplication.
        .send()
        .await?;

    println!("Send message to the queue: {:#?}", rsp);

    Ok(())
}

/// This is the main body for the function.
/// Write your code inside it.
/// You can see more examples in Runtime's repository:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // Extract some useful information from the request
   
     let Opt {
        region,
        queue,
        verbose,
    } = Opt::from_args();
    
    let body = event.body();
    let s = std::str::from_utf8(&body).expect("invalid utf-8 sequence");
    //Log into Cloudwatch
    info!(payload = %s, "JSON Payload received");

    //Serialze JSON into struct.
    //If JSON is incorrect, send back 400 with error.
    let item = match serde_json::from_str::<Item>(s) {
        Ok(item) => item,
        Err(err) => {
            let resp = Response::builder()
                .status(400)
                .header("content-type", "text/html")
                .body(err.to_string().into())
                .map_err(Box::new)?;
            return Ok(resp);
        }
    };

    //Get config from environment.
    let config = aws_config::load_from_env().await;
    //Send to SQS
    let client = Client::new(&config);
    let first_queue_url = find_first_queue(&client).await?;
    let queue_url = queue.unwrap_or(first_queue_url);

    let message = SQSMessage {
        body: "hello from my queue".to_owned(),
        group: "MyGroup".to_owned(),
    };

    send(&client, &queue_url, &message).await?;
 
    //Deserialize into json to return in the Response
    let j = serde_json::to_string(&item)?;

    //Send back a 200 - success
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(j.into())
        .map_err(Box::new)?;
    Ok(resp)
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

// snippet-start:[sqs.rust.sqs-list-first]
async fn find_first_queue(client: &Client) -> Result<String, Error> {
    let queues = client.list_queues().send().await?;
    let queue_urls = queues.queue_urls().unwrap_or_default();
    Ok(queue_urls
        .first()
        .expect("No queues in this account and Region. Create a queue to proceed.")
        .to_string())
}
