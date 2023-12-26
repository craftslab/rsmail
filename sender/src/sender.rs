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

extern crate clap;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::io::Read;

use clap::{Arg, Command};
use lettre::message::{header::ContentType, Attachment, Mailbox, MultiPart};
use lettre::{Message, SmtpTransport, Transport};

#[derive(serde_derive::Deserialize)]
struct Config {
    host: String,
    pass: String,
    port: u16,
    sender: String,
    sep: String,
    user: String,
}

struct Mail {
    attachment: Vec<String>,
    body: String,
    cc: Vec<String>,
    content_type: String,
    from: String,
    subject: String,
    to: Vec<String>,
}

static CONTENT_TYPE_MAP: HashMap<&str, &str> =
    HashMap::from([("HTML", "text/html"), ("PLAIN_TEXT", "text/plain")]);

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

    let default = "".to_string();

    let c = app.get_one("config").unwrap_or(&default);
    let config = parse_config(c)?;

    let a = app.get_one("attachment").unwrap_or(&default);
    let attachment = parse_attachment(&config, a)?;

    let b = app.get_one("body").unwrap_or(&default);
    let body = parse_body(b)?;

    let e = app.get_one("content_type").unwrap_or(&default);
    let content_type = parse_content_type(e)?;

    let header = app.get_one("header").unwrap_or(&default);

    let p = app.get_one("recipients").unwrap_or(&default);
    let (cc, to) = parse_recipients(&config, p);
    if cc.len() == 0 && to.len() == 0 {
        return Err(Box::from("failed to parse recipients"));
    }

    let title = app.get_one("title").unwrap_or(&default);

    let mail = Mail {
        attachment,
        body,
        cc,
        content_type,
        from: header.to_string(),
        subject: title.to_string(),
        to,
    };

    send_mail(&config, &mail)?;

    return Ok(());
}

fn parse_config(name: &String) -> Result<Config, Box<dyn Error>> {
    let mut file = fs::File::open(name)?;
    let mut data = String::new();

    file.read_to_string(&mut data)?;

    return serde_json::from_str(data.as_str()).map_err(|e| e.into());
}

fn parse_attachment(config: &Config, name: &String) -> Result<Vec<String>, Box<dyn Error>> {
    let names = Vec::new();

    if name.is_empty() {
        return Ok(names);
    }

    let mut buf: Vec<String> = name
        .split(&(config.sep).to_owned())
        .map(|s| s.to_string())
        .collect();

    for item in &mut buf {
        *item = check_file(item)?;
    }

    return Ok(buf);
}

fn parse_body(data: &String) -> Result<String, Box<dyn Error>> {
    let buf = match check_file(&data) {
        Ok(b) => b,
        Err(_) => return Ok(data.to_string()),
    };

    match fs::read_to_string(&buf) {
        Ok(b) => Ok(b),
        Err(e) => Err(Box::try_from(e).unwrap()),
    }
}

fn parse_content_type(data: &String) -> Result<String, Box<dyn Error>> {
    match CONTENT_TYPE_MAP.get(data.as_str()) {
        Some(buf) => Ok(buf.to_string()),
        None => Err("content type invalid".into()),
    }
}

fn parse_recipients(config: &Config, data: &String) -> (Vec<String>, Vec<String>) {
    let mut cc: Vec<String> = Vec::new();
    let mut to: Vec<String> = Vec::new();

    for item in data.split(&config.sep) {
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
    cc = collect_difference(cc, to.to_owned());

    return (cc, to);
}

fn send_mail(config: &Config, mail: &Mail) -> Result<(), Box<dyn Error>> {
    let email = Message::builder()
        .from(config.sender.parse()?)
        .to(Mailbox::new(None, mail.to[0].parse().unwrap()))
        .cc(Mailbox::new(None, mail.cc[0].parse().unwrap()))
        .subject(&mail.subject)
        .body(mail.body.clone())
        .unwrap();

    let t = mail.content_type.as_str();
    let content_type = ContentType::parse(t).unwrap();

    let multi_part = MultiPart::builder();

    for item in &mail.attachment {
        let body = fs::read(item)?;
        let attachment = Attachment::new((*item).to_string()).body(body, content_type.to_owned());
        multi_part.clone().singlepart(attachment);
    }

    let creds = lettre::transport::smtp::authentication::Credentials::new(
        config.user.clone(),
        config.pass.clone(),
    );

    let mailer = SmtpTransport::relay(&config.host)
        .unwrap()
        .port(config.port)
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::try_from(e).unwrap()),
    }
}

fn check_file(name: &String) -> Result<String, Box<dyn Error>> {
    let mut buf = name.to_string();

    let metadata = fs::metadata(&name);
    match metadata {
        Ok(md) => {
            if md.is_file() {
                Ok(buf)
            } else {
                Err(Box::try_from("file invalid").unwrap())
            }
        }
        Err(_) => {
            let root = env::current_dir()?;
            let fullname = root.join(name);
            match fs::metadata(&fullname) {
                Ok(md) => {
                    if md.is_file() {
                        buf = fullname.to_str().unwrap().to_string();
                        Ok(buf)
                    } else {
                        Err(Box::try_from("file invalid").unwrap())
                    }
                }
                Err(e) => Err(Box::try_from(e).unwrap()),
            }
        }
    }
}

fn remove_duplicates(data: Vec<String>) -> Vec<String> {
    let mut buf = Vec::new();

    for item in data {
        if !buf.contains(&item) {
            buf.push(item);
        }
    }

    return buf;
}

fn collect_difference(data: Vec<String>, other: Vec<String>) -> Vec<String> {
    let mut buf = Vec::new();
    let mut key = Vec::new();

    for item in other {
        if !key.contains(&item) {
            key.push(item);
        }
    }

    for item in data {
        if !key.contains(&item) {
            buf.push(item);
        }
    }

    return buf;
}
