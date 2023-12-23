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

    print_address(cc, to, filter);

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

    buf = remove_duplicates(buf);

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

    cc = remove_duplicates(cc);
    to = remove_duplicates(to);
    cc = collect_difference(cc, to.to_owned());

    return (cc, to);
}

fn fetch_address(config: &Config, data: Vec<String>) -> Result<Vec<String>, Box<dyn Error>> {
    let fetch = |data: String| -> String {
        let buf: Vec<&str> = data.split("@").collect();
        if buf.len() == 0 {
            "".to_string();
        }
        buf[0].to_string()
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
        Ok(buf.unwrap())
    };

    let mut buf: Vec<String> = vec![];

    for item in data {
        let mut addr = query("mail", item.to_owned())?;
        if addr.is_empty() {
            let a = query("sAMAccountName", fetch(item.to_owned()))?;
            if !a.is_empty() {
                addr = a;
            }
        }
        if !addr.is_empty() {
            buf.push(addr.to_owned())
        }
    }

    Ok(buf)
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

    for item in other {
        if !buf.contains(&item) {
            buf.push(item);
        }
    }

    for item in data {
        if !buf.contains(&item) {
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
