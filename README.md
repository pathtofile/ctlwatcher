# CTL Watcher
Monitor Certificate Transparency logs for domains matching regexes.

# Overview
This project uses [CaliDog's CertStream-Server](https://github.com/CaliDog/certstream-server/issues) to
subscribe to the public lists of new TLS sertificates being recorded in various [Certificate Transparency Logs](https://certificate.transparency.dev) (CTLs).

New domains are checked against a user-supplies list of regexes, outputting matches and optionally sending the matches to Slack via an [incoming webhook](https://api.slack.com/messaging/webhooks).

# Building
Just run:
```bash
# compile project as ./target/debug/ctlwatcher
cargo build
```
Or if on Linux x86_64 grab the binary from from the [latest release](https://github.com/pathtofile/ctlwatcher/releases/latest) page.

# Setup
## CertStream-Server
First start up a local version of CertStream-Server. This is easy using docker:
```bash
# Get code from GitHub
git clone https://github.com/CaliDog/certstream-server
cd certstream-server

# Build docker image
docker build -tag certstream-server:latest .

# Start docker container, opens websocket on localhost:4000
docker run --rm -ti -p 4000:4000 certstream-server:latest
```

## Create Regexes
Then create a file containing regexes to match, one per line, e.g.:
```
ftp
\.com$
[0-9]+apple
```

Regex matching is using [this library](https://docs.rs/regex/latest/regex), which has an implicit `.*` at the start and end of
every pattern, if the `$^` anchors are not used.

## (Optional) Create Slack bot
To send data to slack, create a [Slack Bot](https://api.slack.com/messaging/webhooks), and add it to your server.
Then copy the unique URL to POST the results to, which looks something like:
```
https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXXXXXXXXXXXXXX
```

# Running
```bash
# Where 'regexes.txt' contains list of regexes to match
ctlwatcher --regex-file regexes.txt

# Send results to slack
ctlwatcher --regex-file regexes.txt --slack-url https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXXXXXXXXXXXXXX

# Help and more details
ctlwathcer --help
```
