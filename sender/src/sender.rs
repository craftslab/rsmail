// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use lettre::message::Mailbox;
use lettre::{Message, SmtpTransport, Transport};
use lettre_email::EmailBuilder;
use mime::Mime;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MailError {
    #[error("send failed: {0}")]
    SendFailed(lettre::smtp::error::Error),
    #[error("file invalid")]
    FileInvalid,
    #[error("lstat failed: {0}")]
    LstatFailed(std::io::Error),
}

pub struct Config {
    pub sep: char,
    pub sender: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
}

pub struct Mail {
    pub from: String,
    pub cc: Vec<String>,
    pub subject: String,
    pub to: Vec<String>,
    pub content_type: Mime,
    pub body: String,
    pub attachment: Vec<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let app = Command::new("mail sender")
        .version("1.0.0")
        .author("Jia Jia")
        .arg(
            Arg::new("attachment")
                .long("attachment")
                .short('a')
                .value_name("NAME")
                .help("Attachment files (attach1,attach2)"),
        )
        .arg(
            Arg::new("body")
                .long("body")
                .short('b')
                .value_name("TEXT_OR_NAME")
                .help("Body text or file"),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .value_name("NAME")
                .help("Config file (.json)"),
        )
        .arg(
            Arg::new("content_type")
                .long("content_type")
                .short('e')
                .default_value("PLAIN_TEXT")
                .value_name("TYPE")
                .help("Content type (HTML or PLAIN_TEXT)"),
        )
        .arg(
            Arg::new("header")
                .long("header")
                .short('r')
                .value_name("TEXT")
                .help("Header text"),
        )
        .arg(
            Arg::new("recipients")
                .long("recipients")
                .short('p')
                .value_name("LIST")
                .help("Recipients list (alen@example.com,cc:bob@example.com)")
                .required(true),
        )
        .arg(
            Arg::new("title")
                .long("title")
                .short('t')
                .value_name("TEXT")
                .help("Title text"),
        )
        .get_matches();

    return Ok(());
}

pub fn parse_config() {
}

pub fn parse_attachment() {
}

pub fn parse_body() {
}

pub fn parse_content_type() {
}

pub fn parse_recipients(config: &Config, data: &str) -> (Vec<String>, Vec<String>) {
    let mut cc = vec![];
    let mut to = vec![];

    for item in data.split(config.sep) {
        if !item.is_empty() {
            if item.starts_with("cc:") {
                let buf = item.replace("cc:", "");
                if !buf.is_empty() {
                    cc.push(buf);
                }
            } else {
                to.push(item.to_string());
            }
        }
    }

    cc = remove_duplicates(cc);
    to = remove_duplicates(to);
    cc = collect_difference(cc, to);

    (cc, to)
}

pub fn send_mail(config: &Config, data: &Mail) -> Result<(), MailError> {
    let mut email = EmailBuilder::new()
        .to(data.to.clone())
        .from((config.sender.clone(), data.from.clone()))
        .subject(data.subject.clone())
        .header(("Content-Type", data.content_type.clone()))
        .body(data.body.clone())
        .build()
        .unwrap();

    for item in &data.attachment {
        email = email.attachment(item, None, &mime::APPLICATION_OCTET_STREAM).unwrap();
    }

    let mailer = SmtpTransport::relay(&config.host)
        .unwrap()
        .port(config.port)
        .credentials(lettre::smtp::authentication::Credentials::new(
            config.user.clone(),
            config.pass.clone(),
        ))
        .build();

    mailer.send(&email).map_err(MailError::SendFailed)
}

pub fn check_file(name: &str) -> Result<String, MailError> {
    let path = Path::new(name);
    let metadata = fs::metadata(path).map_err(MailError::LstatFailed)?;

    if !metadata.is_file() {
        return Err(MailError::FileInvalid);
    }

    Ok(name.to_string())
}

fn remove_duplicates(data: Vec<String>) -> Vec<String> {
    let set: HashSet<_> = data.into_iter().collect();
    set.into_iter().collect()
}

fn collect_difference(data: Vec<String>, other: Vec<String>) -> Vec<String> {
    let set: HashSet<_> = other.into_iter().collect();
    data.into_iter().filter(|x| !set.contains(x)).collect()
}
