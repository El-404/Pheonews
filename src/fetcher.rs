use std::error::Error;
use rss::Channel;

pub async fn get_google_channel(subject: &str) -> Result<(Channel, reqwest::StatusCode), Box<dyn Error>> {
    return get_url_channel(&format!("https://news.google.com/rss?q={}&hl=en-US&gl=US&ceid=US%3Aen", subject)).await;
}

pub async fn get_url_channel(url: &str) -> Result<(Channel, reqwest::StatusCode), Box<dyn Error>> {
    let (content, status) = get_url_content(url).await?;
    let channel = content.parse::<Channel>()?;
    Ok((channel, status))
}

pub async fn get_url_content(url: &str) -> Result<(String, reqwest::StatusCode), Box<dyn Error>> {
    let content = reqwest::get(url).await?;
    let status = content.status();

    Ok((content.text().await?, status))
}

pub async fn get_random_article(subject: &str, verbose: bool) -> String {
    let mut article_html;
    'main_loop: loop {
        let channel_result = get_google_channel(subject).await;
        let (channel, status): (Channel, reqwest::StatusCode);
        match channel_result {
            Ok((channel_inner, status_inner)) => {
                (channel, status) = (channel_inner, status_inner);
            },
            Err(e) => { 
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
            Err(e) => {
                if verbose { println!("Failed to decode publisher URL for {}\nURL: {}; retrying", subject, url); }
                continue 'main_loop;
            }
        }
        let content_result = get_url_content(&publisher_url).await;
        match content_result {
            Ok((article_html_inner, article_status)) => {
                article_html = article_html_inner;
                if article_status.is_success() {
                    break 'main_loop;
                } else {
                    if verbose { println!("Failed to get article content for {}\nURL: {}; retrying", subject, publisher_url); }
                    continue 'main_loop;
                }
            },
            Err(e) => {
                if verbose { println!("Failed to get article content for {}\nURL: {}; retrying", subject, publisher_url); }
                continue 'main_loop;
            }
        }
    }
    if verbose { println!("Successfully retrieved article for {}", subject); }
    article_html
}

pub fn parse_html(html: &str) -> String {
    let document = scraper::Html::parse_document(html);
    document.root_element().text().collect::<Vec<_>>().join(" ")
}