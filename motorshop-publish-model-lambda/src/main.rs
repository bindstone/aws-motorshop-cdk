use std::env;
use aws_sdk_dynamodb::{
    types::AttributeValue, Client
};

use lambda_http::{Body, Error, Request, RequestExt, Response, run, service_fn};
use rusoto_core::{Region};
use rusoto_sns::{PublishInput, Sns, SnsClient};
use tracing::info;

use motorshop_domain::prospect::Prospect;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
        .expect("TODO: panic message");
    Ok(())
}

#[tracing::instrument(skip(event), fields(req_id = % event.lambda_context().request_id))]
async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let sns_topic_arn = match env::var_os("SNS_TOPIC_ARN") {
        Some(v) => v.into_string().unwrap(),
        None => panic!("SNS_TOPIC_ARN is not set")
    };

    let sns_topic_name = match env::var_os("SNS_TOPIC_NAME") {
        Some(v) => v.into_string().unwrap(),
        None => panic!("SNS_TOPIC_NAME is not set")
    };

    let dynamo_table = match env::var_os("DYNAMO_TABLE") {
        Some(v) => v.into_string().unwrap(),
        None => panic!("DYNAMO_TABLE is not set")
    };

    let body = event.body();
    let prospect_string = std::str::from_utf8(body).expect("invalid utf-8 sequence");

    //Log into Cloudwatch
    info!(payload = ?prospect_string, "JSON Payload received");

    // Deserialize
    let prospect = match serde_json::from_str::<Prospect>(prospect_string) {
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

    // Store DB
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);

    let request = client
        .put_item()
        .table_name(dynamo_table)
        .item("table_key", AttributeValue::S(String::from("catalog".to_string())))
        .item("data_key", AttributeValue::S(prospect.name.clone()))
        .item("name", AttributeValue::S(prospect.name.clone()))
        .item("model", AttributeValue::S(prospect.model.clone()));
    request.send().await?;

    // SEND SNS Message

    let send_message = serde_json::to_string(&prospect).unwrap();
    info!(send_message = ?send_message, "Send Prospect to SNS");

    let publish_input = PublishInput {
        message: send_message,
        topic_arn: Option::from(sns_topic_arn),
        ..Default::default()
    };

    // Publish the message to the SNS topic
    let region = Region::default();
    let sns_client = SnsClient::new(region);

    let result = sns_client.publish(publish_input).await?;
    info!(result = ?result, "SQS Response");

    let http_response = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(format!("Message put in {}", sns_topic_name).into())
        .map_err(Box::new)?;
    Ok(http_response)
}