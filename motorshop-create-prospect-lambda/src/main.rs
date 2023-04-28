use std::env;
use std::fs::File;
use std::io::Read;

use lambda_runtime::{Error, LambdaEvent, service_fn};
use rusoto_core::{Region, RusotoError};
use rusoto_s3::{PutObjectRequest, S3, S3Client};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::prospect_generator::{generate_prospect, Prospect};

mod prospect_generator;

#[derive(Deserialize)]
struct Request {
    command: String,
}

#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    info!("start...");
    // required to enable CloudWatch error logging by the runtime
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

    let func = service_fn(my_handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

pub(crate) async fn my_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // extract some useful info from the request
    let command = event.payload.command;

    // prepare the response
    let resp = Response {
        req_id: event.context.request_id,
        msg: format!("Command {} executed.", command),
    };

    let mut prospect = Prospect { name: "Duffy".parse().unwrap(), model: "Supra".parse().unwrap() };
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
            println!("File uploaded successfully to S3 bucket");
            Ok(())
        }
        Err(err) => Err(Box::new(RusotoError::from(err))),
    }.expect("TODO: panic message");

    // return `Response` (it will be serialized to JSON automatically by the runtime)
    Ok(resp)
}