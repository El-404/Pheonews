use std::error::Error;
use rss::Channel;
use std::{env,fs::File};
// use mini_redis::{client, Result};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let subject = &args[1];
    let feed = getGoogleSubject(subject).await;

    if feed.is_err() {
        println!("Error fetching feed: {}", feed.unwrap_err());
        return;
    }
    let channel = feed.unwrap();
    
    let mut rng = rand::random::<f64>();
    rng *= channel.items().len() as f64;

    if let Err(e) = open::that(channel.items[rng as usize].link().unwrap()) {
        eprintln!("Failed to open URL: {}", e);
    }
}

async fn getGoogleSubject(subject: &str) -> Result<Channel, Box<dyn Error>> {
    return getRss(&format!("https://news.google.com/rss?q={}&hl=en-US&gl=US&ceid=US%3Aen", subject)).await;
}

async fn getRss(url: &str) -> Result<Channel, Box<dyn Error>> {
let content = reqwest::get(url)
    .await?
    .bytes()
    .await?;

    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}