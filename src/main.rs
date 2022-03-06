use lambda_runtime::{service_fn, Error, LambdaEvent};
use scraper::{Html, Selector};
use serde_json::{json, Value};
use chrono::{DateTime, Utc, Datelike};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let date = event.payload["date"].as_str().expect("No date found");
    println!("Processing date: {}", date);

    let datetime = DateTime::parse_from_rfc3339(date).expect("Failed to parse event date");

    let month = if datetime.month() > 10 {
        datetime.month().to_string()
    } else {
        format!("0{}", datetime.month())
    };
    let day = if datetime.day() > 10 {
        datetime.day().to_string()
    } else {
        format!("0{}", datetime.day())
    };

    let formatted_date = format!(
        "{year}-{month}-{day}",
        year = datetime.year(),
        month = month,
        day = day,
    );
    println!("Processing date: {}", date);
    println!("Formatted date: {}", formatted_date);
    let num_commits_made = get_commits_for_date(formatted_date).await;
    println!("Num commits made: {}", num_commits_made);
    Ok(json!({
        "message": format!("Commits, {}!", num_commits_made)
    }))
}

async fn get_commits_for_date(date: String) -> String {
    let url = "https://github.com/DioneJM";
    let resp = reqwest::get(url).await.expect("Failed to get response");
    debug_assert!(resp.status().is_success());

    let body = resp.text().await.unwrap();
    let fragment = Html::parse_document(&body);

    // date is in format YYYY-MM-DD
    // todo: create type that checks for this
    let selector_query = format!("rect[data-date=\"{}\"]", date);
    let commit_box = Selector::parse(selector_query.as_str()).unwrap();
    let html = fragment
        .select(&commit_box)
        .take(1)
        .next();

    if html.is_none() {
        println!("Could not find commit box for date: {}", selector_query);
        return 0.to_string();
    }

    let html = html.unwrap()
        .value()
        .attr("data-count")
        .unwrap();
    html.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn html_get() {
        let url = "https://github.com/DioneJM";
        let resp = reqwest::get(url).await.expect("Failed to get response");
        assert!(resp.status().is_success());

        let body = resp.text().await.unwrap();
        let fragment = Html::parse_document(&body);
        let username = Selector::parse(".vcard-username").unwrap();
        let html = fragment.select(&username);
        let html = html.take(1).next().unwrap();
        assert_eq!(html.html(),"<span class=\"p-nickname vcard-username d-block\" itemprop=\"additionalName\">\n          DioneJM\n\n        </span>");
    }

    #[tokio::test]
    async fn get_commit() {
        let date = "2022-01-02T18:44:49Z";
        let datetime = DateTime::parse_from_rfc3339(date).expect("Failed to parse event date");
        let month = if datetime.month() > 10 {
            datetime.month().to_string()
        } else {
            format!("0{}", datetime.month())
        };
        let day = if datetime.day() > 10 {
            datetime.day().to_string()
        } else {
            format!("0{}", datetime.day())
        };

        let formatted_date = format!(
            "{year}-{month}-{day}",
            year = datetime.year(),
            month = month,
            day = day,
        );

        let num_commits_made = get_commits_for_date(formatted_date).await;
        assert_eq!(num_commits_made, "8");
    }
}
