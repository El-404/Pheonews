use std::error::Error;
use rss::Channel;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let subject = &args[1];
    let channel = get_google_subject(subject).await?;
    
    let mut rng = rand::random::<f64>();
    rng *= channel.items().len() as f64;
    let url = channel.items[rng as usize].link().unwrap();

    // open::that(url)?;

    let publisher_url = google_news_url_decoder_rs::decode_google_news_url(url).await?;
    println!("{}", publisher_url);
    let _ = std::fs::write("output.html", get_rss_content(&publisher_url).await?);


    Ok(())
}

async fn get_google_subject(subject: &str) -> Result<Channel, Box<dyn Error>> {
    return get_rss_channel(&format!("https://news.google.com/rss?q={}&hl=en-US&gl=US&ceid=US%3Aen", subject)).await;
}

async fn get_rss_channel(url: &str) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get(url)
        .await?
        .bytes()
        .await?;

    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

async fn get_rss_content(url: &str) -> Result<String, Box<dyn Error>> {
    let content = reqwest::get(url)
        .await?
        .text()
        .await?;

    Ok(content)
}