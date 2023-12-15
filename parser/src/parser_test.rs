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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        if parse_config("../config/parser.json").is_err() {
            panic!("FAIL")
        }
    }

    #[test]
    fn test_parse_filter() {
        let config = parse_config("../config/parser.json");
        if config.is_err() {
            panic!("FAIL");
        }

        let filter = "alen@example.com,,bob@example.com,";

        if parse_filter(&config.unwrap(), filter).is_err() {
            panic!("FAIL")
        }
    }

    #[test]
    fn test_parse_recipients() {
        let config = parse_config("../config/parser.json");
        if config.is_err() {
            panic!("FAIL");
        }

        let recipients = "alen@example.com,cc:,cc:bob@example.com,";

        let (cc, to) = parse_recipients(&config.unwrap(), recipients);
        if cc.len() == 0 || to.len() == 0 {
            panic!("FAIL")
        }
    }

    #[test]
    fn test_print_address() {
        let filter = vec!["@example.com"];

        let mut cc = vec!["alen@example.com"];
        let mut to = vec!["bob@example.com"];
        print_address(&cc, &to, &filter);

        to = vec![];
        print_address(&cc, &to, &filter);

        cc = vec![];
        print_address(&cc, &to, &filter);
    }

    #[test]
    fn test_remove_duplicates() {
        let mut buf = vec!["alen@example.com", "bob@example.com", "alen@example.com"];
        buf = remove_duplicates(buf);

        if check_duplicates(&buf) {
            panic!("FAIL")
        }
    }

    #[test]
    fn test_collect_difference() {
        let buf_a = vec!["alen@example.com", "bob@example.com"];
        let buf_b = vec!["alen@example.com"];

        if let buf = collect_difference(&buf_a, &buf_b); buf.len() != 1 {
            panic!("FAIL")
        }
    }

    #[test]
    fn test_filter_address() {
        let filter = vec!["@example.com"];

        let address = "alen@example.com";
        if filter_address(address, &filter).is_err() {
            panic!("FAIL");
        }

        let address = "@example.com";
        if filter_address(address, &filter).is_ok() {
            panic!("FAIL");
        }
    }
}
