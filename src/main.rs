use lambda_runtime::{service_fn, Error, LambdaEvent};
use scraper::{ElementRef, Html, Selector};
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
    let commit_message = get_commit_message_for_date(&formatted_date).await;
    println!("Commit message for date {}: {}", formatted_date, commit_message);

    let sns_client = create_sns_client().await;
    let publish_input = create_publish_input(&commit_message);
    let response = sns_client.publish(publish_input).await;
    match response {
        Ok(_) => println!("Successfully sent text message"),
        Err(e) => println!("Failed to send text message\n{:?}", e.to_string())
    }
    Ok(json!({
        "message": commit_message
    }))
}

fn create_publish_input(message: &String) -> PublishInput {
    PublishInput {
        message: message.to_string(),
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

fn format_error_message(date: &String) -> String {
    String::from(format!("No commits found on {}", date))
}

async fn get_commit_message_for_date(date: &String) -> String {
    let url = "https://github.com/DioneJM";
    let resp = reqwest::get(url).await.expect("Failed to get response");
    debug_assert!(resp.status().is_success());

    let body = resp.text().await.unwrap();
    let fragment = Html::parse_document(&body);

    // date is in format YYYY-MM-DD
    // todo: create type that checks for this
    let selector_query = format!("td[data-date=\"{}\"]", date);
    println!("commit_box query: {}", selector_query);
    let commit_box = Selector::parse(selector_query.as_str()).unwrap();
    let html = fragment
        .select(&commit_box)
        .next();

    if html.is_none() {
        println!("Could not find commit box for date: {}", selector_query);
        return format_error_message(date)
    }

    println!("commit box found, attempting to find number of commits");

    let commit_box_element: ElementRef = html
        .unwrap();
    let commit_box_id = commit_box_element.value().attr("id").expect("no id attr with tooltip id found");
    let tooltip_query = format!("tool-tip[for=\"{}\"]", commit_box_id);
    println!("tooltip query: {}", selector_query);

    let selected_tooltip = Selector::parse(tooltip_query.as_str()).unwrap();
    let tooltip_html = fragment
        .select(&selected_tooltip)
        .next();

    if tooltip_html.is_none() {
        println!("Could not find tooltip for corresponding commit box for date: {}", tooltip_query);
        return format_error_message(date)
    }

    println!("tooltip found, attempting to find number of commits");
    String::from("Commit Checker: ".to_owned() + &tooltip_html.unwrap().inner_html())
}

fn formatted_date_from_rfc3339_timestamp(date: &str) -> String {
    let datetime = DateTime::parse_from_rfc3339(date).expect("Failed to parse event date");

    let month = if datetime.month() >= 10 {
        datetime.month().to_string()
    } else {
        format!("0{}", datetime.month())
    };
    let day = if datetime.day() >= 10 {
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

    #[test]
    fn formats_date_correctly_when_less_than_ten() {
        let date = "2022-01-02T18:44:49Z";
        let formatted_date = formatted_date_from_rfc3339_timestamp(date);
        assert_eq!(formatted_date, "2022-01-02")
    }

    #[test]
    fn formats_date_correctly_when_is_ten() {
        let date = "2022-10-10T18:44:49Z";
        let formatted_date = formatted_date_from_rfc3339_timestamp(date);
        assert_eq!(formatted_date, "2022-10-10")
    }

    #[test]
    fn formats_date_correctly_when_greater_than_ten() {
        let date = "2022-11-11T18:44:49Z";
        let formatted_date = formatted_date_from_rfc3339_timestamp(date);
        assert_eq!(formatted_date, "2022-11-11")
    }

    #[tokio::test]
    async fn get_commit() {
        let date = "2023-11-04T18:44:49Z";
        let formatted_date = formatted_date_from_rfc3339_timestamp(date);
        let commit_message = get_commit_message_for_date(&formatted_date).await;
        assert_eq!(commit_message, String::from("Commit Checker: 6 contributions on November 4th."));
    }

    #[test]
    fn build_publish_input_no_commits() {
        env::set_var(SNS_TOPIC_ARN, "some topic");
        let message = String::from("Commit Checker: this is the message you'll see in your text");
        let publish_input = create_publish_input(&message);
        assert_eq!(publish_input.message, message);
        assert_eq!(publish_input.topic_arn, Some("some topic".to_string()));
    }
}
