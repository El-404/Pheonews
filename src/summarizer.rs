use std::error::Error;
use std::io::Write;
use futures_util::StreamExt;
use reqwest::Client;

#[derive(serde::Deserialize)]
pub struct GeneratedResponse {
    #[serde(default)]
    done: bool,
    #[serde(default)]
    response: String
}
pub async fn summarize(text: &str, n_gen_limit: Option<i32>, client: &Client) -> Result<String, Box<dyn Error>> {
    let sys_prompt = std::fs::read_to_string("sysPrompt.txt")?;
    let input = serde_json::json!({
        "model": "summarizer3",
        "system": sys_prompt,
        "prompt": text,
        "stream": false,
        "options": {
            "num_predict": n_gen_limit.unwrap_or(3000)
        }
    });

    let request = client
        .post("http://localhost:11434/api/generate")
        .json(&input)
        .build()?;

    let response = client.execute(request).await?;
    let json = response.json::<GeneratedResponse>().await?;
    Ok(json.response)
}

pub async fn summarize_stream(text: &str, n_gen_limit: Option<i32>, client: &Client) -> Result<String, Box<dyn Error>> {
    let sys_prompt = std::fs::read_to_string("sysPrompt.txt")?;

    let input = serde_json::json!({
        "model": "summarizer3",
        "system": sys_prompt,
        "prompt": text,
        "stream": true,
        "options": {
            "num_predict": n_gen_limit.unwrap_or(3000)
        }
    });

    let request = client
        .post("http://localhost:11434/api/generate")
        .json(&input)
        .build()?;

    let response = client.execute(request).await?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await?;
        return Err(format!("Ollama returned {status}: {body}").into());
    }

    let mut chunks = response.bytes_stream();
    let mut buffer = Vec::new();
    let mut summary = String::new();

    while let Some(chunk) = chunks.next().await {
        buffer.extend_from_slice(&chunk?);

        while let Some(newline_index) = buffer.iter().position(|byte| *byte == b'\n') {
            let line = buffer.drain(..=newline_index).collect::<Vec<_>>();
            let line = &line[..line.len() - 1];
            let line = line.strip_suffix(b"\r").unwrap_or(line);
            if line.is_empty() {
                continue;
            }

            let generated: GeneratedResponse = serde_json::from_slice(line)?;
            if !generated.response.is_empty() {
                summary.push_str(&generated.response);
                print!("{}", generated.response);
                std::io::stdout().flush()?;
            }

            if generated.done {
                return Ok(summary);
            }
        }
    }

    if summary.is_empty() {
        Err("Ollama stream ended without any generated response".into())
    } else {
        Ok(summary)
    }
}