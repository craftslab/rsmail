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
use std::fs::File;
use std::io::Read;

#[test]
fn test_parse_config() {
    match parse_config("../config/sender.json") {
        Ok(_) => (),
        Err(_) => panic!("FAIL"),
    }
}

#[test]
fn test_parse_attachment() {
    config, err := parseConfig("../config/sender.json")
    if err != nil {
        t.Error("FAIL")
    }

    if _, err := parseAttachment(&config, ""); err != nil {
        t.Error("FAIL")
    }

    if _, err := parseAttachment(&config, "attach1.txt,attach2.txt"); err == nil {
        t.Error("FAIL")
    }

    if _, err := parseAttachment(&config, "../test/attach1.txt,../test/attach2.txt"); err != nil {
        t.Error("FAIL")
    }    
}

#[test]
fn test_parse_body() {
    assert!(parse_body("../test/body.txt").is_ok());
}

#[test]
fn test_parse_content_type() {
    assert!(parse_content_type("FOO").is_err());
    assert!(parse_content_type("HTML").is_err());
    assert!(parse_content_type("PLAIN_TEXT").is_err());
}

#[test]
fn test_parse_recipients() {
    let config = parse_config("../config/sender.json").unwrap();
    let recipients = "alen@example.com,cc:,cc:bob@example.com,";
    let (cc, to) = parse_recipients(&config, recipients);
    assert!(!cc.is_empty() && !to.is_empty());
}

#[test]
fn test_send_mail() {
    let config = parse_config("../config/sender.json").unwrap();
    let mail = Mail {
        attachments: vec!["../test/attach1.txt", "../test/attach2.text"],
        body: "../test/body.txt",
        cc: vec!["catherine@example.com"],
        content_type: "PLAIN_TEXT",
        from: "FROM",
        subject: "SUBJECT",
        to: vec!["alen@example.com, bob@example.com"],
    };
    assert!(send_mail(&config, &mail).is_ok());
}

#[test]
fn test_check_file() {
    assert!(check_file("body.txt").is_err());
    assert!(check_file("test").is_err());
    assert!(check_file("../test/body.txt").is_err());
}

#[test]
fn test_remove_duplicates() {
    let mut buf = vec!["alen@example.com", "bob@example.com", "alen@example.com"];
    buf = remove_duplicates(buf);
    assert!(!check_duplicates(&buf));
}

fn check_duplicates(data: &[String]) -> bool {
    let mut set = HashSet::new();
    for item in data {
        if !set.insert(item) {
            return true;
        }
    }
    false
}

#[test]
fn test_collect_difference() {
    let buf_a = vec!["alen@example.com", "bob@example.com"];
    let buf_b = vec!["alen@example.com"];
    let buf = collect_difference(&buf_a, &buf_b);
    assert_eq!(buf.len(), 1);
}
