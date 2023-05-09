use std::env;
use std::fs::File;
use std::io::Read;

use aws_lambda_events::sqs::{SqsBatchResponse, SqsEventObj};
use lambda_runtime::{Error, LambdaEvent, run, service_fn};
use rusoto_core::{Region, RusotoError};
use rusoto_s3::{PutObjectRequest, S3, S3Client};
use tracing::info;

use motorshop_domain::prospect::Prospect;

use crate::prospect_generator::generate_prospect;

mod prospect_generator;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}

async fn function_handler(sqs_event: LambdaEvent<SqsEventObj<Prospect>>) -> Result<SqsBatchResponse, Error> {
    info!("{:?}", sqs_event);

    let prospect = &sqs_event.payload.records[0].body;

    let doc_name = generate_prospect(prospect.clone());

    let bucket_name = match env::var_os("BUCKET_NAME") {
        Some(v) => v.into_string().unwrap(),
        None => panic!("$BUCKET_NAME is not set")
    };

    let fname = format!("{}-{}.pdf", prospect.name, prospect.model);

    let key_name = format!("prospect/{}", fname);
    let file_name = format!("/tmp/{}", fname);

    let mut file = File::open(file_name)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;

    let s3_client = S3Client::new(Region::default());

    let put_request = PutObjectRequest {
        bucket: bucket_name.to_string(),
        key: key_name,
        body: Some(contents.into()),
        ..Default::default()
    };

    match s3_client.put_object(put_request).await {
        Ok(_) => {
            info!(doc_name = ?doc_name, "File uploaded successfully to S3 bucket");
            Ok(())
        }
        Err(err) => Err(Box::new(RusotoError::from(err))),
    }.expect("TODO: panic message");

    // return `Response` (it will be serialized to JSON automatically by the runtime)
    let resp: SqsBatchResponse = serde_json::from_str("ok").unwrap();

    Ok(resp)
}