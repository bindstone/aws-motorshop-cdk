use lambda_runtime::{Error, LambdaEvent, service_fn};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (event, ctx) = event.into_parts();

    let name = event["body"]["name"].as_str().unwrap_or("unknown").to_uppercase();

    Ok(json!({
        "isBase64Encoded": false,
        "statusCode": 200,
        "headers": {"REQUEST_ID" : ctx.request_id},
        "body": format!("{}", name)
    }))
}