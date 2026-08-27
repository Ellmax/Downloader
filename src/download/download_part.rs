use futures_util::StreamExt;
use reqwest::{Client, header::RANGE};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

use crate::DownloadError;
use crate::part::Part;

pub async fn download_part(client: &Client, url: &str, part: &Part) -> Result<(), DownloadError> {
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
