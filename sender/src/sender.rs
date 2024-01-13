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
use lazy_static::lazy_static;
use lettre::message::{header::ContentType, Attachment, Mailbox, Mailboxes, MultiPart, SinglePart};
use lettre::{Message, SmtpTransport, Transport};

#[derive(serde_derive::Deserialize, Debug)]
struct Config {
    host: String,
    port: u16,
    user: String,
    pass: String,
    sender: String,
    sep: String,
}

#[derive(Debug)]
struct Mail {
    attachment: Vec<String>,
    body: String,
    cc: Vec<String>,
    content_type: String,
    from: String,
    subject: String,
    to: Vec<String>,
}

lazy_static! {
    static ref CONTENT_TYPE_MAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("HTML", "text/html");
        m.insert("PLAIN_TEXT", "text/plain");
        m
    };
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

    let default = "".to_string();

    let mut config = Config {
        host: "localhost".to_string(),
        port: 25,
        user: "".to_string(),
        pass: "".to_string(),
        sender: "sender@example.com".to_string(),
        sep: ",".to_string(),
    };

    let mut mail = Mail {
        attachment: vec![],
        body: "".to_string(),
        cc: vec![],
        content_type: "".to_string(),
        from: "".to_string(),
        subject: "".to_string(),
        to: vec![],
    };

    let c = app.get_one("config").unwrap_or(&default);
    if let Ok(ret) = parse_config(c.as_str()) {
        config = ret;
    }

    let attach = app.get_one("attachment").unwrap_or(&default);
    if let Ok(a) = parse_attachment(&config, attach.as_str()) {
        mail.attachment = a;
    }

    let body = app.get_one("body").unwrap_or(&default);
    if let Ok(b) = parse_body(body.as_str()) {
        mail.body = b;
    }

    let content_type = app.get_one("content_type").unwrap_or(&default);
    if let Ok(c) = parse_content_type(content_type.as_str()) {
        mail.content_type = c;
    }

    let header = app.get_one("header").unwrap_or(&default);
    mail.from = (*header.to_owned()).parse().unwrap();

    let recipients = app.get_one("recipients").unwrap_or(&default);
    let (cc, to) = parse_recipients(&config, recipients.as_str());

    if cc.len() == 0 && to.len() == 0 {
        return Err(Box::from("failed to parse recipients"));
    }

    mail.cc = cc;
    mail.to = to;

    let title = app.get_one("title").unwrap_or(&default);
    mail.subject = (*title.to_owned()).parse().unwrap();

    send_mail(&config, &mail)?;

    return Ok(());
}

fn parse_config(name: &str) -> Result<Config, Box<dyn Error>> {
    let mut file = fs::File::open(name)?;
    let mut data = String::new();

    file.read_to_string(&mut data)?;

    return serde_json::from_str(data.as_str()).map_err(|e| e.into());
}

fn parse_attachment(config: &Config, name: &str) -> Result<Vec<String>, Box<dyn Error>> {
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

fn parse_body(data: &str) -> Result<String, Box<dyn Error>> {
    let buf = match check_file(&data) {
        Ok(b) => b,
        Err(_) => return Ok(data.to_string()),
    };

    match fs::read_to_string(&buf) {
        Ok(b) => Ok(b),
        Err(e) => Err(Box::try_from(e).unwrap()),
    }
}

fn parse_content_type(data: &str) -> Result<String, Box<dyn Error>> {
    match CONTENT_TYPE_MAP.get(data) {
        Some(buf) => Ok(buf.to_string()),
        None => Err("content type invalid".into()),
    }
}

fn parse_recipients(config: &Config, data: &str) -> (Vec<String>, Vec<String>) {
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
    let content_type = ContentType::parse(mail.content_type.as_str())?;
    let body = SinglePart::builder()
        .header(content_type.clone())
        .body(mail.body.clone());
    let multi_part = MultiPart::builder().singlepart(body);

    for item in &mail.attachment {
        let body = fs::read(item)?;
        let attachment = Attachment::new((*item).to_string()).body(body, content_type.clone());
        multi_part.to_owned().singlepart(attachment);
    }

    let mut to = Mailboxes::new();

    for item in mail.to.to_owned() {
        to.push(Mailbox::new(None, item.parse()?))
    }

    let mut cc = Mailboxes::new();

    for item in mail.cc.to_owned() {
        cc.push(Mailbox::new(None, item.parse()?))
    }

    let message = Message::builder()
        .from(Mailbox::new(None, config.sender.parse()?))
        .to(to.into_single().unwrap())
        .cc(cc.into_single().unwrap())
        .sender(Mailbox::new(None, mail.from.parse()?))
        .subject(&mail.subject)
        .multipart(multi_part)?;

    let creds = lettre::transport::smtp::authentication::Credentials::new(
        config.user.clone(),
        config.pass.clone(),
    );

    let mailer = SmtpTransport::relay(&config.host)
        .unwrap()
        .port(config.port)
        .credentials(creds)
        .build();

    match mailer.send(&message) {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::try_from(e).unwrap()),
    }
}

fn check_file(name: &str) -> Result<String, Box<dyn Error>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_parse_config() {
        assert!(parse_config("test/valid.json").is_ok());
        assert!(parse_config("test/invalid.json").is_err());
    }

    #[test]
    fn test_parse_attachment() {
        let config = parse_config("test/valid.json").unwrap();

        match parse_attachment(&config, "") {
            Ok(b) => assert!(b.is_empty()),
            Err(_) => assert!(false),
        }

        let name = "attach1.txt,attach2.txt";
        assert!(parse_attachment(&config, name).is_err());

        let name = "test/attach1.txt,test/attach2.txt";
        match parse_attachment(&config, name) {
            Ok(b) => assert_eq!(b.len(), 2),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_parse_body() {
        assert!(parse_body("").is_ok());

        match parse_body("body") {
            Ok(b) => assert_eq!(b, "body"),
            Err(_) => assert!(false),
        }

        match parse_body("body.txt") {
            Ok(b) => assert_eq!(b, "body.txt"),
            Err(_) => assert!(false),
        }

        assert!(parse_body("test/body.txt").is_ok());
    }

    #[test]
    fn test_parse_content_type() {
        assert!(parse_content_type("FOO").is_err());

        if let Ok(b) = parse_content_type("HTML") {
            assert_eq!(b, "text/html".to_string());
        }

        if let Ok(b) = parse_content_type("PLAIN_TEXT") {
            assert_eq!(b, "text/plain".to_string());
        }
    }

    #[test]
    fn test_parse_recipients() {
        let config = parse_config("test/valid.json").unwrap();

        let recipients = "alen@example.com";
        let (cc, to) = parse_recipients(&config, recipients);
        assert!(cc.is_empty());
        assert_eq!(to.len(), 1);
        assert_eq!(to[0], "alen@example.com");

        let recipients = "alen@example.com,cc:,cc:bob@example.com,";
        let (cc, to) = parse_recipients(&config, recipients);
        assert_eq!(cc.len(), 1);
        assert_eq!(cc[0], "bob@example.com");
        assert_eq!(to.len(), 1);
        assert_eq!(to[0], "alen@example.com");

        let recipients = "alen@example.com,alen@example.com,cc:bob@example.com,cc:bob@example.com,";
        let (cc, to) = parse_recipients(&config, recipients);
        assert_eq!(cc.len(), 1);
        assert_eq!(cc[0], "bob@example.com");
        assert_eq!(to.len(), 1);
        assert_eq!(to[0], "alen@example.com");

        let recipients = "alen@example.com,bob@example.com,cc:bob@example.com,cc:bob@example.com,";
        let (cc, to) = parse_recipients(&config, recipients);
        assert!(cc.is_empty());
        assert_eq!(to.len(), 2);
        assert_eq!(to[0], "alen@example.com");
        assert_eq!(to[1], "bob@example.com");
    }

    #[test]
    fn test_send_mail() {
        assert!(true);
    }

    #[test]
    fn test_check_file() {
        assert!(check_file("body.txt").is_err());
        assert!(check_file("test").is_err());
        assert!(check_file("test/body.txt").is_ok());
    }

    #[test]
    fn test_remove_duplicates() {
        let helper = |data: Vec<String>| -> bool {
            let mut set = HashSet::new();
            for item in data {
                if !set.insert(item) {
                    return true;
                }
            }
            return false;
        };

        let mut buf = vec![
            "alen@example.com".to_string(),
            "bob@example.com".to_string(),
            "alen@example.com".to_string(),
        ];

        buf = remove_duplicates(buf);
        assert!(!helper(buf));
    }

    #[test]
    fn test_collect_difference() {
        let buf_a = vec!["alen@example.com".to_string()];
        let buf_b = vec!["alen@example.com".to_string()];
        let buf = collect_difference(buf_a, buf_b);
        assert!(buf.is_empty());

        let buf_a = vec!["alen@example.com".to_string()];
        let buf_b = vec!["bob@example.com".to_string()];
        let buf = collect_difference(buf_a, buf_b);
        assert_eq!(buf.len(), 1);
        assert_eq!(buf[0], "alen@example.com");

        let buf_a = vec![
            "alen@example.com".to_string(),
            "bob@example.com".to_string(),
        ];
        let buf_b = vec!["alen@example.com".to_string()];
        let buf = collect_difference(buf_a, buf_b);
        assert_eq!(buf.len(), 1);
        assert_eq!(buf[0], "bob@example.com");
    }
}
