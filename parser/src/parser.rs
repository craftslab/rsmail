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
extern crate openssl;

use std::fs::File;
use std::env;
use std::error::Error;
use std::io::Read;

use clap::{Arg, App};
use ldap3::{LdapConn, Scope};
use openssl::ssl::SslConnectorBuilder;
use openssl::ssl::SslMethod;

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
    let app = App::new("parser")
        .version("2.0.7")
        .author("Jia Jia")
        .arg(Arg::with_name("config")
            .short("c")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("filter")
            .short("f")
            .takes_value(true))
        .arg(Arg::with_name("recipients")
            .short("r")
            .takes_value(true)
            .required(true))
        .get_matches();
    let config_name = app.value_of("config").unwrap().to_string();
    let filter_list = app.value_of("filter").unwrap_or("@example1.com,@example2.com").to_string();
    let recipients_list = app.value_of("recipients").unwrap().to_string();

    let config = parse_config(config_name)?;
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
    Ok(data.split(&config.sep).filter(|item| !item.is_empty()).collect())
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
    let mut result = Vec::new();
    let ldap_url = format!("ldap://{}:{}", config.host, config.port);
    let connector = SslConnectorBuilder::new(SslMethod::tls()).unwrap().build();
    let mut ldap = LdapConn::new(&ldap_url, connector)?;

    ldap.simple_bind(&config.user, &config.pass)?;

    for item in data {
        let (search_filter, addr) = if item.contains("@") {
            let search_filter = format!("(&(mail={}))", item);
            let addr = String::from(item);
            (search_filter, addr)
        } else {
            let search_filter = format!("(&(sAMAccountName={}))", item);
            let addr = format!("{}@example.com", item);
            (search_filter, addr)
        };
        let (result, _res) = ldap.search(&config.base, Scope::Subtree, &search_filter, vec!["mail"])?;
        let entry = result.into_iter().next();
        if let Some(entry) = entry {
            let mail = entry.attrs.get("mail").and_then(|ary| ary.first()).map(String::from);
            if mail.is_some() {
                result.push(mail.unwrap());
            } else {
                result.push(addr);
            }
        }
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
        for i in 0..cc.len()-2 {
            print!("cc:{},", cc[i]);
        }

        print!("cc:{}", cc[cc.len()-1]);
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
    data.into_iter().filter(|item| {
        filter.iter().any(|filter|
            item.ends_with(filter)
        )
    }).collect()
}
