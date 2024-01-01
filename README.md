# rsmail

[![Actions Status](https://github.com/craftslab/rsmail/workflows/CI/badge.svg?branch=main&event=push)](https://github.com/craftslab/rsmail/actions?query=workflow%3ACI)
[![License](https://img.shields.io/github/license/craftslab/rsmail.svg?color=brightgreen)](https://github.com/craftslab/rsmail/blob/main/LICENSE)
[![Tag](https://img.shields.io/github/tag/craftslab/rsmail.svg?color=brightgreen)](https://github.com/craftslab/rsmail/tags)



## Introduction

*rsmail* is a mail sender written in Rust.



## Prerequisites

- Rust >= 1.74.0



## Features

*rsmail* supports:

- Attachments
- HTML and text templates



## Config

```bash
bash -c "cat >> ~/.cargo/config" << EOF
[http]
proxy = "host:port"
EOF
```



## Build

```bash
git clone https://github.com/craftslab/rsmail.git

cd rsmail/parser
make install
make build
```

```bash
git clone https://github.com/craftslab/rsmail.git

cd rsmail/sender
make install
make build
```



## Run

```bash
./parser \
  --config="config/parser.json" \
  --filter="@example1.com,@example2.com" \
  --recipients="alen,cc:bob@example.com"
```

```bash
./sender \
  --config="config/sender.json" \
  --attachment="attach1.txt,attach2.txt" \
  --body="body.txt" \
  --content_type="PLAIN_TEXT" \
  --header="HEADER" \
  --recipients="alen@example.com,bob@example.com,cc:catherine@example.com" \
  --title="TITLE"
```



## Usage

```bash
Usage: parser [OPTIONS] --recipients <LIST>

Options:
  -c, --config <NAME>      Config file (.json)
  -f, --filter <LIST>      Filter list (@example1.com,@example2.com)
  -r, --recipients <LIST>  Recipients list (alen,cc:bob@example.com)
  -h, --help               Print help
  -V, --version            Print version
```

```bash
Usage: sender [OPTIONS] --recipients <LIST>

Options:
  -a, --attachment <NAME>    Attachment files (attach1,attach2)
  -b, --body <TEXT_OR_NAME>  Body text or file
  -c, --config <NAME>        Config file (.json)
  -e, --content_type <TYPE>  Content type (HTML or PLAIN_TEXT) [default: PLAIN_TEXT]
  -r, --header <TEXT>        Header text
  -p, --recipients <LIST>    Recipients list (alen@example.com,cc:bob@example.com)
  -t, --title <TEXT>         Title text
  -h, --help                 Print help
  -V, --version              Print version
```



## Test

```bash
# port 25: SMTP
# port 8025: Web UI
docker run --rm --name=mailpit -p 25:1025 -p 8025:8025 axllent/mailpit:latest
```



## License

Project License can be found [here](LICENSE).



## Reference

- [gomail](https://github.com/craftslab/gomail)
- [mailpit](https://mailpit.axllent.org/)
- [mailpit-deploy](https://gist.github.com/craftslab/ae9d77c4e7aa0887f1a8091023d88463)
- [rsproxy](https://rsproxy.cn/)
