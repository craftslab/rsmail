// Licensed under the Apache License, Version 2.0
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software

mod mail;

use mail::Mail;
use std::env;
use serde::{Deserialize, Serialize};
use std::fs;
use std::process;

#[derive(Serialize, Deserialize)]
pub struct Config {
    host: String,
    pass: String,
    port: u32,
    sender: String,
    sep: String,
    user: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = parse_config(&args[1]).unwrap_or_else(|err| {
        eprintln!("Problem parsing argument: {}", err);
        process::exit(1);
    });

    // more code here
}

fn parse_config(file_location: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(file_location)?;
    let config: Config = serde_json::from_str(&contents)?;
    Ok(config)
}

// TODO: FIXME
// msg.Attach(item, mail.Rename(mime.QEncoding.Encode("utf-8", filepath.Base(item))))
