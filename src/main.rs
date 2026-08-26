use reqwest::StatusCode;
use std::process::exit;
use std::{fs::File, io::Write};
use thiserror::Error;

use futures_util::StreamExt;
use reqwest::{Client, Response, header::CONTENT_DISPOSITION};

#[derive(Error, Debug)]
enum DownloadError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Invalid Content-Disposition header")]
    InvalidContentDisposition,

    #[error("No filename found in URL or headers")]
    MissingFilename,

    #[error("HTTP error: {0}")]
    HttpStatus(StatusCode),
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: dlr <url>");
        exit(2)
    }

    let d: Result<(), DownloadError> = download(&args[1]).await;

    match d {
        Ok(..) => println!("ok"),
        Err(e) => {
            eprintln!("{}", e);
            exit(1)
        }
    }
}

async fn download(url: &str) -> Result<(), DownloadError> {
    let client: Client = Client::new();
    let response: Response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(DownloadError::HttpStatus(response.status()));
    }

    let header = response
        .headers()
        .get(CONTENT_DISPOSITION)
        .and_then(|val| val.to_str().ok())
        .ok_or(DownloadError::InvalidContentDisposition)?;

    let filename = header
        .split("filename=")
        .nth(1)
        .ok_or(DownloadError::MissingFilename)?;

    // let size = response.content_length().ok_or_else(|| anyhow!("Ошибка("))?;

    let mut file = File::create(filename)?;

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
    }
    Ok(())
}
