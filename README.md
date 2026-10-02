 # Pheonews

Pheonews is a news fetcher and summarizer that collects articles using google news rss.

## Features

- Fetch a list of articles related to a subject
- Parse an article and extract text
- Summarize news and present biases

## Dependencies

- [Ollama](https://ollama.com/)
    - Pheonews uses an Ollama agent to summarize the parsed article
    - Install a model and rename it summarizer
    - Serve the model on localhost:11434

- [google-news-url-decoder-rs](https://github.com/El-404/google-news-url-decoder-rs)
    - Converts google news links into the links for their respective article
    - Vibe coded by converting [google-news-url-decoder](https://github.com/SSujitX/google-news-url-decoder) into rust

- [Rust](https://rust-lang.org/)
    - I think this one is obvious

## Usage

To fetch a random article based on a <subject> use the command

```cargo run <subject>```
