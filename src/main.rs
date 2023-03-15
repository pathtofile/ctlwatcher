use clap::Parser;
use futures_util::StreamExt;
use regex::RegexSet;
use serde_json::{json, Value};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use tokio::io::AsyncWriteExt;
use tokio_tungstenite::connect_async;

// Setup Commandline args
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// List of regexes, one per line
    #[arg(short, long, default_value = "regexes.txt")]
    regex_file: String,

    /// Certstream Websocket URL, e.g. 'wss://certstream.calidog.io/' or 'ws://127.0.0.1:4000'
    #[arg(short, long, default_value = "ws://127.0.0.1:4000/")]
    url: String,

    /// Debug printing
    #[arg(short, long)]
    debug: bool,
}

async fn report_matches(matches: &Vec<String>, domain: &str) -> Result<(), Box<dyn Error>> {
    for m in matches {
        let data = json!({
            "regex": m,
            "domain": domain
        });
        let text = format!("{}\n", data);
        tokio::io::stdout().write_all(text.as_bytes()).await?
    }
    Ok(())
}

async fn check_json(data: &str, set: &RegexSet) -> Result<(), Box<dyn Error>> {
    let v: Value = serde_json::from_str(data)?;

    let t = v["message_type"].as_str().ok_or("Missing message_type")?;
    if t == "certificate_update" {
        // Extract domains from JSON
        let domains = v
            .pointer("/data/leaf_cert/all_domains")
            .ok_or("JSON Missing all_domains")?
            .as_array()
            .ok_or("JSON all_domains not an array")?;

        // Check for any matches
        for domain in domains {
            let domain = domain.as_str().ok_or("domain not a string")?;
            let matches: Vec<String> = set
                .matches(domain)
                .into_iter()
                .map(|m| set.patterns()[m].clone())
                .collect();
            if !matches.is_empty() {
                report_matches(&matches, domain).await?;
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse args
    let args = Args::parse();

    // Read and compile regexes from file
    let lines: Vec<String> =
        BufReader::new(File::open(args.regex_file).expect("Cannot open regex file"))
            .lines()
            .map(|l| l.unwrap())
            .filter(|l| !l.is_empty())
            .collect();
    let set = RegexSet::new(lines).expect("failed to compile regexes from file");

    loop {
        // Connect to websocket stream
        let stream = connect_async(&args.url).await.expect("Failed to connect").0;

        // Parse certificates concurrently
        stream
            .for_each(|msg| async {
                let set = set.clone();
                tokio::spawn(async move {
                    if let Ok(data) = msg.and_then(|msg| msg.into_text()) {
                        if let Err(e) = check_json(&data, &set).await {
                            eprintln!("Error! {}", e)
                        }
                    }
                });
            })
            .await;
    }
    // Ok(())
}
