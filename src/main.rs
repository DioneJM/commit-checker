use lambda_runtime::{service_fn, Error, LambdaEvent};
use scraper::{Html, Selector};
use serde_json::{json, Value};
use chrono::{DateTime, Datelike};
use rusoto_core::{Region};
use rusoto_sns::{Sns, SnsClient, PublishInput};
use std::env;

const SNS_TOPIC_ARN: &str = "SNS_TOPIC_ARN";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let date = event.payload["date"].as_str().expect("No date found");
    println!("Processing date: {}", date);

    let formatted_date = formatted_date_from_rfc3339_timestamp(date);

    println!("Processing date: {}", date);
    println!("Formatted date: {}", formatted_date);
    let num_commits_made = get_commits_for_date(&formatted_date).await;
    println!("Num commits made: {}", num_commits_made);

    let sns_client = create_sns_client().await;
    let publish_input = create_publish_input(num_commits_made, &formatted_date);
    let response = sns_client.publish(publish_input).await;
    match response {
        Ok(_) => println!("Successfully sent text message"),
        Err(e) => println!("Failed to send text message\n{:?}", e.to_string())
    }
    Ok(json!({
        "message": format!("Commits, {}!", num_commits_made)
    }))
}

fn create_publish_input(num_commits_made: i32, date: &String) -> PublishInput {
    let message = if num_commits_made > 1 {
        format!("Nice job! You made {} commits on {}", num_commits_made, date)
    } else if num_commits_made == 1 {
        format!("Nice job on the commit you made on {}", date)
    } else {
        format!("You haven't made a commit yet today 😢 Make sure to have a commit in!")
    };

    PublishInput {
        message,
        message_attributes: None,
        message_deduplication_id: None,
        message_group_id: None,
        message_structure: None,
        phone_number: None,
        target_arn: None,
        topic_arn: Some(env::var(SNS_TOPIC_ARN).expect("Failed to find SNS topic ARN")),
        subject: Some("Commit Check".to_string()),
    }
}

async fn create_sns_client() -> SnsClient {
    SnsClient::new(Region::ApSoutheast2)
}

async fn get_commits_for_date(date: &String) -> i32 {
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
        return 0;
    }

    let html = html.unwrap()
        .value()
        .attr("data-count")
        .unwrap();
    html.to_string().parse::<i32>().unwrap_or(0)
}

fn formatted_date_from_rfc3339_timestamp(date: &str) -> String {
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

    format!(
        "{year}-{month}-{day}",
        year = datetime.year(),
        month = month,
        day = day,
    )
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
        assert!(html.html().contains("DioneJM"));
    }

    #[tokio::test]
    async fn get_commit() {
        let date = "2022-01-02T18:44:49Z";
        let formatted_date = formatted_date_from_rfc3339_timestamp(date);
        let num_commits_made = get_commits_for_date(&formatted_date).await;
        assert_eq!(num_commits_made, 8);
    }

    #[test]
    fn build_publish_input_no_commits() {
        env::set_var(SNS_TOPIC_ARN, "some topic");
        let commits = 0;
        let date = "2022-01-01".to_string();
        let publish_input = create_publish_input(commits, &date);
        assert_eq!(publish_input.message, "You haven't made a commit yet today 😢 Make sure to have a commit in!");
        assert_eq!(publish_input.topic_arn, Some("some topic".to_string()));
    }

    #[test]
    fn build_publish_input_single_commit() {
        env::set_var(SNS_TOPIC_ARN, "some topic");
        let commits = 1;
        let date = "2022-01-01".to_string();
        let publish_input = create_publish_input(commits, &date);
        assert_eq!(publish_input.message, format!("Nice job on the commit you made on {}", date));
        assert_eq!(publish_input.topic_arn, Some("some topic".to_string()));
    }

    #[test]
    fn build_publish_input_more_than_one_commit() {
        env::set_var(SNS_TOPIC_ARN, "some topic");
        let commits = 2;
        let date = "2022-01-01".to_string();
        let publish_input = create_publish_input(commits, &date);
        assert_eq!(publish_input.message, format!("Nice job! You made {} commits on {}", commits, date));
        assert_eq!(publish_input.topic_arn, Some("some topic".to_string()));
    }
}
