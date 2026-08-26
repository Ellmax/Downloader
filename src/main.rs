use std::{fs::File, io::Write};

// use anyhow::anyhow;

use reqwest::{Client, Response, header::{self, CONTENT_DISPOSITION}};
use futures_util::{StreamExt, future::err};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Usage: dlr <url>")
    }

    let d : Result<(), anyhow::Error> = download(&args[1]).await;

    match d {
        Ok(..) => println!("ok"),
        Err(e) => println!("err: {}", e)
    }
}

async fn download(url: &str) -> Result<(), anyhow::Error>{
    let client: Client = Client::new();
    let response: Response = client.get(url).send().await?;
    
    let header = response.headers()
        .get(CONTENT_DISPOSITION)
        .and_then(|val| val.to_str().ok())
        .unwrap_or("no header");

    let filename = header.split("filename=").nth(1).unwrap_or("no filename");

    // let size = response.content_length().ok_or_else(|| anyhow!("Ошибка("))?;

    let mut file = File::create(filename)?;

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
    };
    Ok(())
}