use std::error::Error;
use std::env;

mod summarizer;
mod fetcher;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let subject = &args[1];
    let verbose = args.len() > 2 && args[2] == "--verbose";

    let article_html = fetcher::get_random_article(subject, verbose).await;
    let summarized_response = summarizer::summarize(&fetcher::parse_html(&article_html)).await;

    
    Ok(())
}