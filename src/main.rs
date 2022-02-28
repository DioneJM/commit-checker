use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())}

async fn handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let first_name = event.payload["firstName"].as_str().expect("No first name found");
    Ok(json!({ "message": format!("Waddup, {}!", first_name) }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn handler_handles() {
        let event = json!({
            "answer": 42
        });
        assert_eq!(
            handler(LambdaEvent::new(event.clone(), Context::default()))
                .await
                .expect("expected Ok(_) value"),
            event
        )
    }
}
