use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{json, Value};
use scraper::{Html, Selector};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())}

async fn handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let date = event.payload["date"].as_str().expect("No date found");
    let num_commits_made = get_commit_box_for_day(date.to_string()).await;
    Ok(json!({ "message": format!("Commits, {}!", num_commits_made) }))
}

async fn get_commit_box_for_day(date: String) -> String {
    let url = "https://github.com/DioneJM";
    let resp = reqwest::get(url).await.expect("Failed to get response");
    debug_assert!(resp.status().is_success());

    let body = resp.text().await.unwrap();
    let fragment = Html::parse_document(&body);

    // date is in format YYYY-MM-DD
    let selector_query = format!("rect[data-date=\"{}\"", date);
    let commit_box = Selector::parse(selector_query.as_str()).unwrap();
    let html = fragment
        .select(&commit_box)
        .take(1)
        .nth(0)
        .unwrap()
        .value()
        .attr("data-count")
        .unwrap();
    html.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_runtime::Context;
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

    #[tokio::test]
    async fn html_get() {
        let url = "https://github.com/DioneJM";
        let mut resp = reqwest::get(url).await.expect("Failed to get response");
        assert!(resp.status().is_success());

        let body = resp.text().await.unwrap();
        let fragment = Html::parse_document(&body);
        let username = Selector::parse("rect[data-date=\"2022-02-28\"").unwrap(); //Selector::parse(".vcard-username").unwrap();
        let html = fragment.select(&username);
        let html = html.take(1).nth(0).unwrap();
        assert_eq!(html.html(),"<span class=\"p-nickname vcard-username d-block\" itemprop=\"additionalName\">\n          DioneJM\n\n        </span>");
    }

    #[tokio::test]
    async fn get_commit() {
        let date = "2022-01-02";
        let num_commits_made = get_commit_box_for_day(date.to_string()).await;
        assert_eq!(num_commits_made, "8");
    }
}
