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
use ldap3::{LdapConn, Scope, SearchEntry};

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
                .help("Config file (.json)")
                .required(true),
        )
        .arg(
            Arg::new("filter")
                .long("filter")
                .short('f')
                .value_name("LIST")
                .help("Filter list (@example1.com,@example2.com)")
                .required(true),
        )
        .arg(
            Arg::new("recipients")
                .long("recipients")
                .short('r')
                .value_name("LIST")
                .help("Recipients list (alen,cc:bob@example.com")
                .required(true),
        )
        .get_matches();

    let config_file: String;
    let filter_list: String;
    let recipients_list: String;

    match app.get_one::<String>("config") {
        Some(name) => config_file = name.to_string(),
        None => config_file = "".to_string(),
    }

    match app.get_one::<String>("filter_list") {
        Some(list) => filter_list = list.to_string(),
        None => filter_list = "".to_string(),
    }

    match app.get_one::<String>("recipients_list") {
        Some(list) => recipients_list = list.to_string(),
        None => recipients_list = "".to_string(),
    }

    let config = parse_config(config_file)?;
    let filter = parse_filter(&config, filter_list)?;
    let (cc, to) = parse_recipients(&config, recipients_list);

    let cc = fetch_address(&config, cc)?;
    let to = fetch_address(&config, to)?;

    print_address(cc, to, &filter);

    Ok(())
}

fn parse_config(name: String) -> Result<Config, Box<dyn Error>> {
    let mut file = File::open(name)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    serde_json::from_str(data.as_str()).map_err(|e| e.into())
}

fn parse_filter(config: &Config, data: String) -> Result<Vec<String>, Box<dyn Error>> {
    let mut buf: Vec<String> = vec![];

    for item in data.split(&config.sep) {
        if !item.is_empty() {
            if item.starts_with("@") {
                buf.push(item.to_string());
            }
        }
    }

    Ok(buf)
}

fn parse_recipients(config: &Config, data: String) -> (Vec<String>, Vec<String>) {
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

    (remove_duplicates(cc), remove_duplicates(to))
}

fn fetch_address(config: &Config, data: Vec<String>) -> Result<Vec<String>, Box<dyn Error>> {
    let ldap_url = format!("ldap://{}:{}", config.host, config.port);
    let mut addr: String;
    let mut filter: String;
    let mut ldap = LdapConn::new(&ldap_url)?;
    let mut result: Vec<String> = vec![];

    ldap.simple_bind(&config.user, &config.pass)?;

    for item in data {
        if item.contains("@") {
            addr = String::from(item.clone());
            filter = format!("(&(mail={}))", item.clone());
        } else {
            addr = format!("{}@example.com", item.clone());
            filter = format!("(&(sAMAccountName={}))", item.clone());
        };
        let (rs, _res) = ldap
            .search(&config.base, Scope::Subtree, &filter, vec!["mail"])?
            .success()?;
        for entry in rs {
            let mail = SearchEntry::construct(entry)
                .attrs
                .get("mail")
                .and_then(|ary| ary.first())
                .map(String::from);
            if mail.is_some() {
                result.push(mail.unwrap());
            } else {
                result.push(addr.clone());
            }
        }
        ldap.unbind()?;
    }

    Ok(result)
}

fn print_address(cc: Vec<String>, to: Vec<String>, filter: &Vec<String>) {
    let cc = filter_addresses(remove_duplicates(cc), filter);
    let to = filter_addresses(remove_duplicates(to), filter);

    for address in &to {
        print!("{},", address);
    }

    if !cc.is_empty() {
        for i in 0..cc.len() - 2 {
            print!("cc:{},", cc[i]);
        }

        print!("cc:{}", cc[cc.len() - 1]);
    }

    println!();
}

fn remove_duplicates(data: Vec<String>) -> Vec<String> {
    let mut v = Vec::new();
    for item in data {
        if !v.contains(&item) {
            v.push(item);
        }
    }
    v
}

fn filter_addresses(data: Vec<String>, filter: &Vec<String>) -> Vec<String> {
    data.into_iter()
        .filter(|item| filter.iter().any(|filter| item.ends_with(filter)))
        .collect()
}
