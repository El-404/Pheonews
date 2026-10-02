use std::error::Error;
use reqwest::Response;
use rss::Channel;
use std::env;

#[derive(serde::Deserialize)]
struct GeneratedResponse {
  response: String
}

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
    
    std::fs::write("input.html", &article_html)?;
    let out = summarize(&article_html).await?;

    let json = out.json::<GeneratedResponse>().await?;
    // println!("Model: {}", json.model);
    // println!("Created at: {}", json.created_at);
    // println!("Response: {}", json.response);
    std::fs::write("output.md", json.response)?;
    // std::fs::write("output.md", out.text().await?)?;

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


async fn summarize(text: &str) -> Result<Response, Box<dyn Error>> {
    let sys_prompt = std::fs::read_to_string("sysPrompt.txt")?;
    let client = reqwest::Client::new();

    let request = serde_json::json!({
        "model": "summarizer3",
        "prompt": sys_prompt + text,
        "stream": false,
        // "format": {
            // "Title": "string",
            // "properties": {
                // "paragraph": {
                    // "markdown": "string"
                // },
                // "biases": "string"
            // },
            // "required": ["paragraph", "biases"]
        // },
        "options": {
          "num_ctx": 32768
        }
    });

    let request = client
        .post("http://localhost:11434/api/generate")
        .json(&request)
        .build()?;

    let response = client.execute(request).await?;
    Ok(response)
}