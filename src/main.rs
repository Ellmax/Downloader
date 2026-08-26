use reqwest::{
    StatusCode,
    header::{ACCEPT_RANGES, CONTENT_LENGTH},
};
use std::path::PathBuf;
use std::process::exit;
use thiserror::Error;

use futures_util::{StreamExt, stream};
use reqwest::{
    Client,
    header::{CONTENT_DISPOSITION, RANGE},
};
use tokio::fs::File;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

use content_disposition::parse_content_disposition;

#[derive(Error, Debug)]
enum DownloadError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    HttpStatus(StatusCode),
}

struct Part {
    start: u64,
    end: Option<u64>,
    temp_path: PathBuf,
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: dlr <url>");
        exit(2)
    }

    let d: Result<(), DownloadError> = download(&args[1], 2).await; // я уберу хардкод кол-ва частей, честно...

    match d {
        Ok(..) => println!("ok"),
        Err(e) => {
            eprintln!("{}", e);
            exit(1)
        }
    }
}

async fn download(url: &str, num_parts: usize) -> Result<(), DownloadError> {
    let client = reqwest::Client::builder().user_agent("dlr/0.1.0").build()?;
    let head_response = client.head(url).send().await?;

    if !head_response.status().is_success() {
        return Err(DownloadError::HttpStatus(head_response.status()));
    }

    let size = head_response.headers().get(CONTENT_LENGTH);

    let header = head_response
        .headers()
        .get(CONTENT_DISPOSITION)
        .and_then(|v| v.to_str().ok());

    let mut filename: Option<String> = if let Some(header) = header {
        parse_content_disposition(header).filename_full()
    } else {
        None
    };

    if filename.is_none() {
        filename = head_response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .map(|s| s.into())
    }

    let filename = filename.unwrap_or_else(|| "downloaded_file".into());

    let accept_ranges = head_response
        .headers()
        .get(ACCEPT_RANGES)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("bytes"))
        .unwrap_or(false);

    let use_parralel = size.is_some() && accept_ranges && num_parts > 1;

    let parts: Vec<Part> = if use_parralel {
        let size: u64 = size.unwrap().to_str().unwrap().parse().unwrap();
        let part_size = size / num_parts as u64;
        (0..num_parts)
            .map(|i| {
                let start = i as u64 * part_size;
                let end = if i == num_parts - 1 {
                    Some(size - 1)
                } else {
                    Some(start + part_size - 1)
                };
                let temp_path = format!("{}.part{}", filename, i).into();
                Part {
                    start,
                    end,
                    temp_path,
                }
            })
            .collect::<Vec<Part>>()
    } else {
        vec![Part {
            start: 0,
            end: None,
            temp_path: format!("{}.part0", filename).into(),
        }]
    };

    let concurrency: usize = if use_parralel { num_parts } else { 1 };
    let part_futures = parts.iter().map(|part| download_part(&client, url, part));

    let results: Vec<Result<(), DownloadError>> = stream::iter(part_futures)
        .buffer_unordered(concurrency)
        .collect()
        .await;

    for result in results {
        result?
    }

    let mut final_file = File::create(&filename).await?;
    for part in parts {
        let mut part_file = File::open(&part.temp_path).await?;
        tokio::io::copy(&mut part_file, &mut final_file).await?;
        tokio::fs::remove_file(&part.temp_path).await?;
    }

    Ok(())
}

async fn download_part(client: &Client, url: &str, part: &Part) -> Result<(), DownloadError> {
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(&part.temp_path)
        .await?;

    let size = file.metadata().await?.len();
    let start = part.start + size;

    let mut request = client.get(url);

    if let Some(end) = part.end {
        request = request.header(RANGE, format!("bytes={}-{}", start, end))
    } else if size > 0 {
        file.set_len(0).await?;
    }

    let response = request.send().await?;

    if !response.status().is_success() {
        return Err(DownloadError::HttpStatus(response.status()));
    }

    file.seek(std::io::SeekFrom::Start(size)).await?;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
    }

    Ok(())
}
