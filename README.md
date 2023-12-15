# gomail

[![Actions Status](https://github.com/craftslab/gomail/workflows/CI/badge.svg?branch=master&event=push)](https://github.com/craftslab/gomail/actions?query=workflow%3ACI)
[![Go Report Card](https://goreportcard.com/badge/github.com/craftslab/gomail)](https://goreportcard.com/report/github.com/craftslab/gomail)
[![License](https://img.shields.io/github/license/craftslab/gomail.svg?color=brightgreen)](https://github.com/craftslab/gomail/blob/master/LICENSE)
[![Tag](https://img.shields.io/github/tag/craftslab/gomail.svg?color=brightgreen)](https://github.com/craftslab/gomail/tags)



## Introduction

*gomail* is a mail sender written in Go.



## Prerequisites

- Go >= 1.16.0



## Features

*gomail* supports:

- Attachments
- HTML and text templates



## Build

```bash
git clone https://github.com/craftslab/gomail.git

cd gomail
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
usage: parser --recipients=RECIPIENTS [<flags>]

Recipient parser

Flags:
      --help                   Show context-sensitive help (also try --help-long
                               and --help-man).
      --version                Show application version.
  -c, --config=CONFIG          Config file, format: .json
  -f, --filter=FILTER          Filter list, format: @example1.com,@example2.com
  -r, --recipients=RECIPIENTS  Recipients list, format: alen,cc:bob@example.com
```

```bash
usage: sender --recipients=RECIPIENTS [<flags>]

Mail sender

Flags:
      --help                     Show context-sensitive help (also try
                                 --help-long and --help-man).
      --version                  Show application version.
  -a, --attachment=ATTACHMENT    Attachment files, format: attach1,attach2,...
  -b, --body=BODY                Body text or file
  -c, --config=CONFIG            Config file, format: .json
  -e, --content_type=PLAIN_TEXT  Content type, format: HTML or PLAIN_TEXT
                                 (default)
  -r, --header=HEADER            Header text
  -p, --recipients=RECIPIENTS    Recipients list, format:
                                 alen@example.com,cc:bob@example.com
  -t, --title=TITLE              Title text
```



## License

Project License can be found [here](LICENSE).
