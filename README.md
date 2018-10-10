[![Build Status](https://travis-ci.org/poanetwork/poa-governance-notifications.svg?branch=master)](https://travis-ci.org/poanetwork/poa-governance-notifications) 

# `poa-governance-notifications`

A tool to monitor a POA Network blockchain for
[governance events](https://github.com/poanetwork/wiki/wiki/Governance-Overview).

The `poagov` command line tool is distributed as a binary for Linux and
OSX; it can also be built from source for both platforms.

You can find the source code for the currently deployed governance contracts
[here](https://github.com/poanetwork/poa-network-consensus-contracts/tree/master/contracts).

You can find the addresses for governance contracts currently deployed to Core
[here](https://github.com/poanetwork/poa-chain-spec/blob/core/contracts.json)
and Sokol
[here](https://github.com/poanetwork/poa-chain-spec/blob/sokol/contracts.json).

# Installing the `poagov` Binary

*Note:* the `poagov` binary requires libssl to be installed prior to
usage, if you do not have libssl installed, go to the "Requires libssl"
section in this README to find out how to download it.  

On Debian/Ubuntu:

    $ curl -OL https://github.com/poanetwork/poa-governance-notifications/releases/download/v1.0.0/poagov-1.0.0-linux-x86_64.tar.gz
    $ tar -xvzf poagov-1.0.0-linux-x86_64.tar.gz
    $ rm poagov-1.0.0-linux-x86_64.tar.gz
    $ cd poagov
    $ cp sample.env .env
    $ chmod +x poagov
    $ ./poagov --help

On OSX:

    $ curl -OL https://github.com/poanetwork/poa-governance-notifications/releases/download/v1.0.0/poagov-1.0.0-osx-x86_64.tar.gz
    $ tar -xvzf poagov-1.0.0-osx-x86_64.tar.gz
    $ rm poagov-1.0.0-osx-x86_64.tar.gz
    $ cd poagov
    $ cp sample.env .env
    $ chmod +x poagov
    $ ./poagov --help


# Building `poagov` from Source

To build the `poagov` CLI tool, run the following:

    $ git clone https://github.com/poanetwork/poa-governance-notifications.git
    $ cd poa-governance-notifications
    $ cargo build --release

`poagov` can be built using Rust 1.29 stable and requires `libssl` to be
installed; see the following "Requires libssl" section for more information.

Building `poagov` requires Rust `1.29.0-stable` or later and `libssl`; see the
"Requires `libssl`" section for more information.

### Testing

You can run `poagov`'s tests to ensure that it everything is working properly:

    $ cargo test --release

The test suite will verify: that the required env-vars are found the `.env`
file, that each network's JSON-RPC server can be reached, and that each
contract ABI can be loaded.

# Requires `libssl`

SMTP over TLS requires that you have a native TLS library installed on your
machine, the preferred library for Linux and OSX is OpenSSL >= 1.0.1,
otherwise known as `libssl` (you will need more than just the OpenSSL
binary that you may or may not already have installed at
`/usr/bin/openssl`).

If running `cargo build --release` panics with an error like:

    "error: failed to run custom build command for `openssl-sys v0.9.28
    ...
    Could not find directory of OpenSSL installation
    ..."

you probably do not have `libssl` installed.

To install `libssl` on Debian/Ubuntu run the following:

    $ sudo apt update
    $ sudo apt-get install -y pkg-config libssl-dev

To install libssl on MacOS run the following:

    $ brew update
    $ brew install openssl

Then try to rebuild `poagov` using:

    $ cargo clean
    $ cargo build --release

If you are on OSX and installed OpenSSL using Homebrew and continue to get
compilation errors for any of the Rust crates: `openssl`, `openssl-sys`, or
`openssl-sys-extras`, try building with the following:

    $ cargo clean
    $ OPENSSL_INCLUDE_DIR=$(brew --prefix openssl)/include \
          OPENSSL_LIB_DIR=$(brew --prefix openssl)/lib \
          cargo build --release

There is a known issue regarding the `openssl-sys` crate not being able to
find `libssl` installed with Homebrew on OSX; more information can be found on
[Stack Overflow](https://stackoverflow.com/questions/34612395/openssl-crate-fails-compilation-on-mac-os-x-10-11/34615626#34615626).
The above solution comes from the linked Stack Overflow thread.

More information on common issues encountered while installing the
`openssl` Rust crate can be found [here](https://crates.io/crates/openssl).

# Usage

Once you have built or downloaded `poagov`, you can print out the CLI usage by
running:

    # If you downloaded the `poagov` binary run:
    $ poagov --help
    # Or, if you built `poagov` from source run:
    $ target/release/poagov --help

    poagov 1.0.0
    Monitors a POA Network blockchain for governance events.

    USAGE:
        poagov [FLAGS] [OPTIONS]

    FLAGS:
            --core             monitor voting contracts deployed to the Core network
            --earliest         begin monitoring for governance events starting at the first block in the blockchain
            --email            enables email notifications (SMTP configurations must be set in your `.env` file)
        -e, --emission         monitors the blockchain for ballots to manage emission funds
        -h, --help             prints help information
        -k, --keys             monitors the blockchain for ballots to change keys
            --latest           begin monitoring for governance events starting at the last block mined
        -p, --proxy            monitors the blockchain for ballots to change the proxy address
            --sokol            monitor voting contracts deployed to the Sokol network
        -t, --threshold        monitors the blockchain for ballots to change the minimum threshold
            --v1               monitors the v1 voting contracts
            --v2               monitors the v2 voting contracts
        -V, --version          prints version information
            --log-emails       logs each notification's email body; does not require the --email flag to be set
            --log-file         logs are written to files in the ./logs directory, log files are rotated when they reach a size of 4MB

    OPTIONS:
            --block-time <value>    the average number of seconds it takes to mine a new block
        -n, --limit <value>         shutdown `poagov` after this many notifications have been generated, useful when testing
            --start <value>         start monitoring for governance events at this block (inclusive)
            --tail <value>          start monitoring for governance events for the `n` blocks prior to the last block mined

Hitting `[ctrl-c]` while `poagov` is running will cause the process to gracefully shutdown.

### Required Arguments

Each time you run `poagov`, four CLI arguments are required:

1. The chain (specify only one): `--core`, `--sokol`.
2. The contracts to monitor (specify at least one): `--keys`, `--threshold`, `--proxy`, `--emission`.
2. The hardfork version (specify only one): `--v1`, `--v2`.
4. The block in the chain from where to start monitoring (specify only one): `--earliest`, `--latest`, `--start=<block_number>`, `--tail=<value>`.

### Notes on the hardfork version options `--v1` and `--v2`

`--v1` indicates that you want to monitor for governance events prior to the
Sokol and Core hardforks that will occur in September-2018 and November-2018
respectively.

`--v2` indicates that you want to monitor for governance events that occured
after the above hardfork dates.

- More information regarding the planned hardforks for the Sokol and Core
chains in September and November 2018 can be found
[here](https://medium.com/poa-network/poa-network-news-and-updates-36-2e6e00550c15).

### Optional Arguments

Providing the `--email` flag will enable governance notification emails. To use
this option, you must first configure SMTP in your `.env`.

Providing the `--block-time=<value>` will set how often `poagov` will query the
blockchain for new governance events. Defaults to 30 seconds.

Providing the `--log-emails` flag will print the full text for a notification
email to stderr when governance events are found. When this option is set,
email text will be logged regardless of whether or not the `--email` flag is
set.

Setting the `--log-file` flag will write logs to a file in the `./logs/`
directory. Logs are rotated chronologically across three files. Once the
`logs` directory has reached its max number of files, the oldest log file will
be deleted to make room for the next log file. Log files have a max size of
4MB; the log files will rotated once the current log file has reached the max
file size.

Setting the `--limit=<value>` option will cause `poagov` to stop once `value`
number of notifications have been generated. This option is useful when testing.

# Setting up the `.env` File

When the `poagov` CLI tool is run, the process' environment variables are
loaded via an `.env` file.

The `.env` file contains configuration variables that are not specified via the
command line. You are required to have an `.env` file in the same directory as
your `Cargo.toml` or `poagov` binary.

When building from source, the `sample.env` file will be copied into the `.env`
file. This `.env` file will contain the default configuration values required
to run `poagov`.

If you did not build `poagov` from source, you will have to create an `.env`
file in the same directory as the `poagov` binary; then copy the contents of
`sample.env` into it.

If you wish to enable email notifications, you must add the required SMTP
config values to your `.env` file. See the "Setting up Email Notifications"
section for details.

### Setting up Email Notifications

In order to enable email notifications, you must change the name of the
`sample.env` file to `.env`. Then, you must add values for the following
SMTP config options in your `.env` file:

    EMAIL_RECIPIENTS=
    SMTP_HOST_DOMAIN=
    SMTP_PORT=
    SMTP_USERNAME=
    SMTP_PASSWORD=
    OUTGOING_EMAIL_ADDRESS=

Add a comma-separated list of email address to the "VALIDATORS" config
option in your .env file. These addresses will be sent emails when `poagov`
encounters governance events on the POA blockchain.

*Note* `poagov` forces SMTP email notifcations to be sent over an encrypted
channel, if your SMTP Host does not support TLS or STARTTLS, `poagov` will
panic. You may notice that we default `SMTP_PORT` to port 587 for STARTTLS,
but you may use port 465 for TLS, or any other port that your outgoing
email server is lisening for secure connections. If you require unencrypted
SMTP, submit an issue and I can add it.

Your SMTP configuration should look something like the following:

    EMAIL_RECIPIENTS=alice@poa.network,bob@poa.network
    SMTP_HOST_DOMAIN=mail.riseup.net
    SMTP_PORT=587
    SMTP_USERNAME=evariste_galois
    SMTP_PASSWORD='finteFIELDS#$!'
    OUTGOING_EMAIL_ADDRESS=evariste_galois@riseup.net

# An Explained Example

    $ poagov --sokol --v1 -kt --earliest --email --log-emails --limit=1

- `--sokol` monitors the Sokol chain.
- `--v1` monitors the governance contracts deployed prior to September-2018.
- `-k` monitors the `VotingToChangeKeys` contract.
- `-t` monitors the `VotingToChangeMinThreshold` contract.
- `--earliest` start monitoring from the first block in the blockchain.
- `--email` sends out email notifications to each address in the `EMAIL_RECIPIENTS` env-var.
- `--log-emails` for each governance notification generated, log the corresponding email body.
- `--limit=1` stop running `poagov` after one ballot notification has been generated.

# Logs

Logs are output to `stderr` unless the `--log-file` CLI flag is set. Events
that are logged include: the generation of governance notifications, sending an
email successesfully or failing to send an email, aned what range of blocks
from the chain have been successfully monitored for governance events.
Optionally, you can log the email body for each governance notification
generated by setting the `--log-emails` CLI flag.

The following is an example command with its corresponding logs:

    $ poagov --sokol --v1 --threshold --earliest --limit=3

    Oct 10 15:18:09.863 INFO starting poagov...
    Oct 10 15:18:10.287 INFO governance notification, block_number: 525296, ballot_id: 0, ballot: Threshold
    Oct 10 15:18:10.287 INFO governance notification, block_number: 599789, ballot_id: 1, ballot: Threshold
    Oct 10 15:18:10.287 INFO governance notification, block_number: 1078816, ballot_id: 2, ballot: Threshold
    Oct 10 15:18:10.287 WARN reached notification limit, gracefully shutting down..., limit: 3

