mod download;
mod error;
mod part;

use std::process::exit;

use clap::Parser;

use download::download;
use error::DownloadError;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(short, long, default_value_t = 1)]
    parts: usize,

    url: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let d: Result<(), DownloadError> = download(&args.url, args.parts).await;

    match d {
        Ok(..) => println!("ok"),
        Err(e) => {
            eprintln!("{}", e);
            exit(1)
        }
    }
}
