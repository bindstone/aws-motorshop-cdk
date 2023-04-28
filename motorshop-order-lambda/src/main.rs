use lambda_http::{Body, Error, Request, RequestExt, Response, run, service_fn};
use tracing::info;

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

    run(service_fn(function_handler)).await;
    Ok(())
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Item {
    pub name: String,
}

#[tracing::instrument(skip(event), fields(req_id = % event.lambda_context().request_id))]
async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let body = event.body();
    let s = std::str::from_utf8(body).expect("invalid utf-8 sequence");

    //Log into Cloudwatch
    info!(payload = %s, "JSON Payload received");

    // Deserialize
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

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(format!("{}", item.name.to_uppercase()).into())
        .map_err(Box::new)?;
    Ok(resp)
}