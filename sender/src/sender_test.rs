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

extern crate tempfile;

use std::collections::HashSet;

#[test]
fn test_parse_config() {
    assert!(parse_config("../config/sender.json").is_ok());
}

#[test]
fn test_parse_attachment() {
    let config = parse_config("../config/sender.json").unwrap();

    assert!(parse_attachment(&config, "").is_ok());
    assert!(parse_attachment(&config, "attach1.txt,attach2.txt").is_err());
    assert!(parse_attachment(&config, "../test/attach1.txt,../test/attach2.txt").is_err());
}

#[test]
fn test_parse_body() {
    assert!(parse_body("").is_ok());
    assert!(parse_body("body").is_ok());
    assert!(parse_body("body.txt").is_err());
    assert!(parse_body("../test/body.txt").is_err());
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

    let cc_and_to = parse_recipients(&config, recipients);

    assert!(!cc_and_to.0.is_empty() && !cc_and_to.1.is_empty());
}

#[test]
fn test_send_mail() {
    let config = parse_config("../config/sender.json").unwrap();

    let mail = Mail {
        attachments: vec!["../test/attach1.txt".into(), "../test/attach2.text".into()],
        body: "../test/body.txt".into(),
        recipients: vec!["catherine@example.com".into()],
        content_type: "PLAIN_TEXT".into(),
        from: "FROM".into(),
        subject: "SUBJECT".into(),
        cc_and_to: vec!["alen@example.com".into(), "bob@example.com".into()],
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
    let mut buf = vec!["alen@example.com".into(), "bob@example.com".into(), "alen@example.com".into()];
    let buf_set: HashSet<_> = buf.drain(..).collect(); // remove duplicates
    buf.extend(buf_set.into_iter());

    assert!(!check_duplicates(&buf));
}

#[test]
fn test_collect_difference() {
    let buf_a: HashSet<_> = vec!["alen@example.com", "bob@example.com"].into_iter().collect();
    let buf_b: HashSet<_> = vec!["alen@example.com"].into_iter().collect();

    assert_eq!(buf_a.difference(&buf_b).count(), 1);
}

fn check_duplicates(data: &[String]) -> bool {
    let data_set: HashSet<_> = data.iter().collect();
    data.len() != data_set.len()
}
