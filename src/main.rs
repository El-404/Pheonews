use std::error::Error;
use reqwest::Client;
use rss::Channel;
use std::env;

mod summarizer;


async fn get_random_article(subject: &str, verbose: bool, client: &Client) -> String {
    let mut article_html;
    'main_loop: loop {
        let channel_result = get_google_channel(subject, client).await;
        let (channel, status): (Channel, reqwest::StatusCode);
        match channel_result {
            Ok((channel_inner, status_inner)) => {
                (channel, status) = (channel_inner, status_inner);
                if !status.is_success() {
                    if verbose { println!("Failed to get search result for {}; retrying", subject); }
                    continue 'main_loop;
                }
            },
            Err(_e) => { 
                if verbose { println!("Failed to get search result for {}; retrying", subject); }
                continue 'main_loop;
            }
        }

        let rng = rand::random::<f64>() * channel.items().len() as f64;
        let url = channel.items[rng as usize].link().unwrap();

        let publisher_url_result = google_news_url_decoder_rs::decode_google_news_url(url).await;
        let publisher_url: String;
        match publisher_url_result {
            Ok(publisher_url_inner) => {
                publisher_url = publisher_url_inner;
            },
            Err(_e) => {
                if verbose { println!("Failed to decode publisher URL for {}\nURL: {}; retrying", subject, url); }
                continue 'main_loop;
            }
        }
        let content_result = get_url_content(&publisher_url, client).await;
        match content_result {
            Ok((article_html_inner, article_status)) => {
                article_html = article_html_inner;
                if article_status.is_success() {
                    break 'main_loop;
                } else {
                    if verbose { println!("Failed to get article content for {} at:\n{}; retrying", subject, publisher_url); }
                    continue 'main_loop;
                }
            },
            Err(_e) => {
                if verbose { println!("Failed to get article content for {} at:\n{}; retrying", subject, publisher_url); }
                continue 'main_loop;
            }
        }
    }
    if verbose { println!("Successfully retrieved article for {}", subject); }
    article_html
}

fn parse_html(html: &str) -> String {
    let document = scraper::Html::parse_document(html);
    let content_selector = scraper::Selector::parse("article, h1, h2, h3, h4, h5, span, p").unwrap();

    let mut text = String::new();
    for element in document.select(&content_selector) {
        text += &(element.text().collect::<Vec<_>>().join(" ") + "\n");
    }

    text
}

async fn get_google_channel(subject: &str, client: &Client) -> Result<(Channel, reqwest::StatusCode), Box<dyn Error>> {
    return get_url_channel(&format!("https://news.google.com/rss?q={}&hl=en-US&gl=US&ceid=US%3Aen", subject), client).await;
}

async fn get_url_channel(url: &str, client: &Client) -> Result<(Channel, reqwest::StatusCode), Box<dyn Error>> {
    let (content, status) = get_url_content(url, client).await?;
    let channel = content.parse::<Channel>()?;
    Ok((channel, status))
}

async fn get_url_content(url: &str, client: &Client) -> Result<(String, reqwest::StatusCode), Box<dyn Error>> {
    let content = client.get(url).send().await?;
    let status = content.status();

    Ok((content.text().await?, status))
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let subject = &args[1];
    let verbose = args.len() > 2 && args[2] == "--verbose";
    let client = reqwest::Client::new();

    let article_html = get_random_article(subject, verbose, &client).await;
    let parsed_html = parse_html(&article_html);
    let _ = std::fs::write("input", &parsed_html);
    let summary = summarizer::summarize_stream(&parsed_html, Some(7000), &client).await?;
    std::fs::write("output.md", summary)?;

    Ok(())
}