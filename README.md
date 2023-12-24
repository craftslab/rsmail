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
Usage: parser --config <NAME> --filter <LIST> --recipients <LIST>

Options:
  -c, --config <NAME>      Config file (.json)
  -f, --filter <LIST>      Filter list (@example1.com,@example2.com)
  -r, --recipients <LIST>  Recipients list (alen,cc:bob@example.com)
  -h, --help               Print help
  -V, --version            Print version
```

```bash
TBD
```



## License

Project License can be found [here](LICENSE).
