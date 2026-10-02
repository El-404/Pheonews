use std::{error::Error, process::Output};
use rss::Channel;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let subject = &args[1];

    let (channel, _) = get_google_channel(subject).await?;

    let mut article_html: String;
    loop {
        let mut rng = rand::random::<f64>();
        rng *= channel.items().len() as f64;
        let url = channel.items[rng as usize].link().unwrap();

        let publisher_url = google_news_url_decoder_rs::decode_google_news_url(url).await?;

        let article_status: reqwest::StatusCode;
        (article_html, article_status) = get_url_content(&publisher_url).await?;

        if article_status == reqwest::StatusCode::OK { break; }
    }
    
    std::fs::write("/tmp/input.html", &article_html)?;
    let out = summarize()?;
    print!("{}", String::from_utf8_lossy(&out.stdout));

    Ok(())
}

async fn get_google_channel(subject: &str) -> Result<(Channel, reqwest::StatusCode), Box<dyn Error>> {
    return get_url_channel(&format!("https://news.google.com/rss?q={}&hl=en-US&gl=US&ceid=US%3Aen", subject)).await;
}

async fn get_url_channel(url: &str) -> Result<(Channel, reqwest::StatusCode), Box<dyn Error>> {
    let (content, status) = get_url_content(url).await?;
    let channel = content.parse::<Channel>()?;
    Ok((channel, status))
}

async fn get_url_content(url: &str) -> Result<(String, reqwest::StatusCode), Box<dyn Error>> {
    let content = reqwest::get(url).await?;
    let status = content.status();

    Ok((content.text().await?, status))
}


fn summarize() -> Result<Output, std::io::Error> {
    std::process::Command::new("./runLLM.sh").output()
}