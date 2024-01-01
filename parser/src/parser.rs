// Licensed under the Apache License, Version 2.0 (the "License");
// You may not use this file except in compliance with the License.
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
extern crate ldap3;

use std::error::Error;
use std::fs::File;
use std::io::Read;

use clap::{Arg, Command};
use ldap3::{LdapConn, LdapConnSettings, Scope, SearchEntry};

#[derive(serde_derive::Deserialize)]
struct Config {
    base: String,
    host: String,
    pass: String,
    port: u16,
    sep: String,
    user: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let app = Command::new("recipient parser")
        .version("1.0.0")
        .author("Jia Jia")
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .value_name("NAME")
                .help("Config file (.json)"),
        )
        .arg(
            Arg::new("filter")
                .long("filter")
                .short('f')
                .value_name("LIST")
                .help("Filter list (@example1.com,@example2.com)"),
        )
        .arg(
            Arg::new("recipients")
                .long("recipients")
                .short('r')
                .value_name("LIST")
                .help("Recipients list (alen,cc:bob@example.com)")
                .required(true),
        )
        .get_matches();

    let default = "".to_string();

    let c = app.get_one("config").unwrap_or(&default);
    let config = parse_config(c.as_str())?;

    let f = app.get_one("filter").unwrap_or(&default);
    let filter = parse_filter(&config, f.as_str())?;

    let r = app.get_one("recipients").unwrap_or(&default);
    let (mut cc, mut to) = parse_recipients(&config, r.as_str());
    if cc.len() == 0 && to.len() == 0 {
        return Err(Box::from("failed to parse recipients"));
    }

    cc = fetch_address(&config, cc)?;
    to = fetch_address(&config, to)?;

    print_address(cc, to, filter);

    return Ok(());
}

fn parse_config(name: &str) -> Result<Config, Box<dyn Error>> {
    let mut file = File::open(name)?;
    let mut data = String::new();

    file.read_to_string(&mut data)?;

    return serde_json::from_str(data.as_str()).map_err(|e| e.into());
}

fn parse_filter(config: &Config, data: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut buf: Vec<String> = vec![];

    if data.is_empty() {
        return Ok(buf);
    }

    for item in data.split(&config.sep) {
        if !item.is_empty() {
            if item.starts_with("@") {
                buf.push(item.to_string());
            }
        }
    }

    buf = remove_duplicates(buf);

    return Ok(buf);
}

fn parse_recipients(config: &Config, data: &str) -> (Vec<String>, Vec<String>) {
    let mut cc = Vec::new();
    let mut to = Vec::new();

    for item in data.split(&config.sep) {
        if !item.is_empty() {
            if item.starts_with("cc:") {
                let recipient = item.trim_start_matches("cc:");
                if !recipient.is_empty() {
                    cc.push(recipient.to_owned());
                }
            } else {
                to.push(item.to_owned());
            }
        }
    }

    cc = remove_duplicates(cc);
    to = remove_duplicates(to);
    cc = collect_difference(cc, to.to_owned());

    return (cc, to);
}

fn fetch_address(config: &Config, data: Vec<String>) -> Result<Vec<String>, Box<dyn Error>> {
    let fetch = |data: String| -> String {
        let buf: Vec<&str> = data.split("@").collect();
        if buf.len() == 0 {
            return "".to_string();
        }
        return buf[0].to_string();
    };

    let query = |filter: &str, data: String| -> Result<String, Box<dyn Error>> {
        let mut ldap: LdapConn = LdapConn::with_settings(
            LdapConnSettings::new()
                .set_no_tls_verify(true)
                .set_starttls(true),
            &format!("ldap://{}:{}", config.host, config.port),
        )?;
        ldap.simple_bind(&config.user, &config.pass)?;
        let (entry, _res) = ldap
            .search(
                &config.base,
                Scope::Subtree,
                &format!("(&({}={}))", filter, data),
                vec!["*"],
            )?
            .success()?;
        if entry.len() == 0 {
            return Err(Box::from("failed to search"));
        }
        let buf = SearchEntry::construct(entry[0].to_owned())
            .attrs
            .get("mail")
            .and_then(|ary| ary.first())
            .map(String::from);
        ldap.unbind()?;
        return Ok(buf.unwrap());
    };

    let mut buf: Vec<String> = vec![];

    for item in data {
        let mut addr = "".to_string();
        match query("mail", item.to_owned()) {
            Ok(a) => addr = a,
            Err(_) => {
                if let Ok(a) = query("sAMAccountName", fetch(item.to_owned())) {
                    addr = a;
                }
            }
        }
        if !addr.is_empty() {
            buf.push(addr.to_owned())
        }
    }

    return Ok(buf);
}

fn print_address(cc: Vec<String>, to: Vec<String>, filter: Vec<String>) {
    let mut cc = remove_duplicates(cc);
    let to = remove_duplicates(to);

    cc = collect_difference(cc, to.to_owned());

    for item in to {
        if let Ok(()) = filter_address(item.to_owned(), filter.to_owned()) {
            print!("{},", item);
        }
    }

    if cc.is_empty() {
        return;
    }

    for i in 0..cc.len() - 1 {
        if let Ok(()) = filter_address(cc[i].to_owned(), filter.to_owned()) {
            print!("cc:{},", cc[i]);
        }
    }

    if let Ok(()) = filter_address(cc[cc.len() - 1].to_owned(), filter.to_owned()) {
        println!("cc:{}", cc[cc.len() - 1]);
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

fn filter_address(data: String, filter: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut res = Err("filter failed".into());

    for item in filter {
        if data.ends_with(item.as_str()) {
            if data != item {
                res = Ok(());
            }
            break;
        }
    }

    return res;
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
    fn test_parse_filter() {
        let config = parse_config("test/valid.json").unwrap();

        assert!(parse_filter(&config, "").is_ok());

        let filter = "@example.com";
        if let Ok(b) = parse_filter(&config, filter) {
            assert_eq!(b.len(), 1);
            assert_eq!(b[0], "@example.com");
        }

        let filter = "@example.com,alen@example.com";
        if let Ok(b) = parse_filter(&config, filter) {
            assert_eq!(b.len(), 1);
            assert_eq!(b[0], "@example.com");
        }

        let filter = "@example1.com,,@example1.com,";
        if let Ok(b) = parse_filter(&config, filter) {
            assert_eq!(b.len(), 1);
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
    fn test_fetch_address() {
        assert!(true);
    }

    #[test]
    fn test_print_address() {
        let filter = vec!["@example.com".to_string()];

        let cc = vec!["alen@example.com".to_string()];
        let to = vec!["bob@example.com".to_string()];
        print_address(cc.clone(), to.clone(), filter.clone());

        let to = vec![];
        print_address(cc.clone(), to.clone(), filter.clone());

        let cc = vec![];
        print_address(cc.clone(), to.clone(), filter.clone());
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

    #[test]
    fn test_filter_address() {
        let filter = vec!["@example.com".to_string()];

        let address = "alen@example.com".to_string();
        assert!(filter_address(address, filter.clone()).is_ok());

        let address = "@example.com".to_string();
        assert!(filter_address(address, filter.clone()).is_err());
    }
}
