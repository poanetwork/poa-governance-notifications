[![Build Status](https://travis-ci.org/poanetwork/poa-governance-notifications.svg?branch=master)](https://travis-ci.org/poanetwork/poa-governance-notifications) 

# `poa-governance-notifications`

A CLI tool for monitoring a blockchain for POA Network governance ballots. This tool can be used to
monitor _any_ chain that uses POA Network's governance contracts.

More info regarding governance can be found in
[POA Network's Wiki](https://github.com/poanetwork/wiki/wiki/Governance-Overview).

POA Network's governance contracts can be found in the
[`poa-network-consensus-contracts` repo](https://github.com/poanetwork/poa-network-consensus-contracts/tree/master/contracts),
all Solidity files prefixed with "Voting" are classified as a "governance contract".

The `poagov` command line tool is distributed as a binary for Linux and
OSX. The `poagov` binary can be built from source for both OSX and Linux using the code in this repo.

### Downloading the `poagov` Binary

*Note:* the `poagov` binary requires `libssl` to be installed prior to
usage, if you do not have `libssl` installed, go to the "Requires libssl"
section in this README to find out how to download it.  

    # Download `poagov` for Debian/Ubuntu:
    $ curl -OL https://github.com/poanetwork/poa-governance-notifications/releases/download/v2.0.0/poagov-2.0.0-linux-x86_64.tar.gz
    $ tar -xvzf poagov-2.0.0-linux-x86_64.tar.gz
    $ rm poagov-2.0.0-linux-x86_64.tar.gz
    $ cd poa-governance-notifications
    $ mv sample.env .env
    # Optionally rename binary from `poagov-2.0.0-linux-x86_64` to `poagov`:
    $ mv poagov-2.0.0-linux-x86_64 poagov

    # Or download `poagov` for OSX:
    $ curl -OL https://github.com/poanetwork/poa-governance-notifications/releases/download/v2.0.0/poagov-2.0.0-osx-x86_64.tar.gz
    $ tar -xvzf poagov-2.0.0-osx-x86_64.tar.gz
    $ rm poagov-2.0.0-osx-x86_64.tar.gz
    $ cd poa-governance-notifications
    $ mv sample.env .env
    # Optionally rename binary from `poagov-2.0.0-osx-x86_64` to `poagov`:
    $ mv poagov-2.0.0-osx-x86_64 poagov

    # If you did not rename your binary in the previous step, replace "poagov"
    # in the following commands with your Linux or OSX binary's name:
    $ chmod +x poagov
    $ ./poagov --help

### Building `poagov` from Source

To build the `poagov` CLI tool from source, clone this repo and run:

    $ cargo build --release

`poagov` can be built with Rust 1.31.0-stable or later and requires `libssl`;
see the "Running and Building `poagov` Requires `libssl`" section in this README for more information.

##### Testing `poagov`

You can run `poagov`'s tests to ensure that everything is working properly:

    $ cargo test --release

### Running and Building `poagov` Requires `libssl`

SMTP over TLS requires that you have a native TLS library installed on your
machine, the preferred library for Linux and OSX is OpenSSL >= 1.0.1,
otherwise known as `libssl`.

If running `cargo build --release` panics with an error like:

    "error: failed to run custom build command for `openssl-sys v0.9.28
    ...
    Could not find directory of OpenSSL installation
    ..."

you probably do not have `libssl` installed.

You can use `dpkg` to check to see if you have `libssl` installed, and if so
which version(s) are installed:

    $ dpkg -l | grep libssl

To install `libssl` on Debian/Ubuntu run the following:

    $ sudo apt update
    $ sudo apt-get install -y pkg-config libssl-dev

To install `libssl` on MacOS run the following:

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

### Usage

Once you have built or downloaded `poagov`, you can print out the CLI usage by
running:

    # If you downloaded the `poagov` binary run:
    $ poagov --help
    # Or, if you built `poagov` from source run:
    $ target/release/poagov --help

    poagov 2.0.0
    Monitors a POA Network blockchain for governance events.

    USAGE:
        poagov [FLAGS] [OPTIONS]

    FLAGS:
            --core          Monitors POA Network's Core Network for governance ballots
            --sokol         Monitors POA Network's Sokol network for governance ballots
            --xdai          Monitors the xDai Network for governance ballots

            --v1            Monitors the v1 governance contracts
            --v2            [default] Monitors the v2 governance contracts, if no contract version CLI argument is given by

        -k, --keys          Monitors the blockchain for ballots to change keys
        -t, --threshold     Monitors the blockchain for ballots to change the minimum threshold
                            the user, we set this CLI flag
        -p, --proxy         Monitors the blockchain for ballots to change the proxy address
        -e, --emission      Monitors the blockchain for ballots to manage emission funds

            --earliest      Monitor for governance events starting at the blockchain's first block
            --latest        Monitor for governance events starting at the blockchain's most recently mined block

            --email         Enables email notifications (SMTP configuration options must be set in your `.env` file)
            --log-emails    Logs the full email body for each notification generated, this option does not require the
                            `--email` flag to be set
            --log-file      Logs are written to files in the ./logs directory, logs are rotated chronologically across 3
                            files, each file has a max size of 4MB

        -h, --help          Prints help information
        -V, --version       Prints version information

    OPTIONS:
            --block-time <value>    The average number of seconds it takes to mine a new block
        -n, --limit <value>         Stops `poagov` after this many notifications have been generated (this option can be
                                    useful when testing `poagov`)

            --start <value>         Start monitoring for governance events at this block (inclusive)
            --tail <value>          Start monitoring for governance events for the `n` blocks prior to the last mined block

Hitting `[ctrl-c]` while `poagov` is running will cause the process to gracefully shutdown.

##### Required CLI Arguments

Each time you run `poagov`, three CLI arguments are required:

1. The chain (specify only one): `--core`, `--sokol`, `--xdai`.
2. The governance ballots to monitor (specify at least one): `--keys`, `--threshold`, `--proxy`, `--emission`.
3. The block in the chain from where to start monitoring (specify only one): `--earliest`, `--latest`, `--start=<block_number>`, `--tail=<value>`.

##### Notes on the Hardfork Version CLI Options: `--v1` and `--v2`

`--v1` indicates that you want to monitor for governance events prior to the
Sokol and Core hardforks that will occur in September-2018 and November-2018
respectively.

`--v2` indicates that you want to monitor for governance events that occurred
after the above hardfork dates.

We default to `--v2` being set as it will monitor the currently deployed contract.

- More information regarding the planned hardforks for the POA Sokol and Core
chains in September and November 2018 can be found
[here](https://medium.com/poa-network/poa-network-news-and-updates-36-2e6e00550c15).

### Optional Arguments

Providing the `--v1` flag will tell `poagov` to look for ballots corresponding
to the hardfork #1 governance contracts. The hardfork #1 contracts are not
currently being by POA Network and not new governance notifications will be
generated, however you can use `poagov` to view all past `--v1` ballots that
have occurred using:

	$ poagov <--core, --sokol> --v1 --earliest -ktp

Providing the `--email` flag will enable governance notification via email. To
use this option, you must first configure SMTP in your `.env` file.

Providing the `--block-time=<value>` will set how often `poagov` will query the
blockchain for new governance events. Defaults to 30 seconds.

Providing the `--log-emails` flag will print the full text for a notification
email to `stderr` when governance events are found. When this option is set,
email text will be logged regardless of whether or not the `--email` flag is
set.

Setting the `--log-file` flag will write logs to a file in the `logs/`
directory. Logs are rotated chronologically across three files. Once the
`logs/` directory has reached its max number of files, the oldest log file will
be deleted to make room for the next log file. Log files have a max size of
4MB; the log files will rotated once the current log file has reached the max
file size.

Setting the `--limit=<value>` option will cause `poagov` to stop once `value`
number of notifications have been generated. This option is useful when testing.

### Setting up the `.env` File

When the `poagov` CLI tool is run, the process' environment variables are
loaded via an `.env` file. The `.env` file contains configuration variables
that are not specified via the command line. You are required to have an `.env`
file in the same directory as your `Cargo.toml` or `poagov` binary.

If you downloaded a `.tar.gz` compressed archive containing the `poagov` binary
and you do not have an `.env` file in the unarchived directory, manually copy
the `sample.env` file in this repo into a file called `.env` in the same
directory as the `poagov` binary.

When building from source, the `sample.env` file will be copied into the `.env`
file.

The default `.env` file will contain the default configuration values required
to run `poagov`. If you wish to enable email notifications, you must add the
required SMTP config values to your `.env` file. See the "Setting up Email
Notifications" section for details.

##### Setting up Email Notifications

In order to enable email notifications, you must change the name of the
`sample.env` file to `.env`. Then, you must add values for the following
SMTP config options in your `.env` file:

    SMTP_HOST_DOMAIN=
    SMTP_PORT=
    SMTP_USERNAME=
    SMTP_PASSWORD=
    OUTGOING_EMAIL_ADDRESS=
    EMAIL_RECIPIENTS=

Add a comma-separated list of email address to the `EMAIL_RECIPIENTS` config
option in your `.env` file. These addresses will be sent emails when `poagov`
encounters new ballots.

*Note* `poagov` forces SMTP email notifcations to be sent over TLS/STARTTLS, if
your SMTP Host does not support TLS or STARTTLS, `poagov` will `panic!`.

You may notice that we default `SMTP_PORT` to port 587 for STARTTLS, but you
may use any port for which your outgoing email server is listening; port 465 is
commonly used for TLS. 

Your SMTP configuration should look something like the following:

    SMTP_HOST_DOMAIN=mail.riseup.net
    SMTP_PORT=587
    SMTP_USERNAME=evariste_galois
    SMTP_PASSWORD='finteFIELDS#$!'
    OUTGOING_EMAIL_ADDRESS=evariste_galois@riseup.net
    EMAIL_RECIPIENTS=alice@poa.network,bob@poa.network

### An Explained Example

    $ poagov --sokol --v1 -kt --earliest --email --log-emails --limit=1

- `--sokol` monitors the Sokol chain.
- `--v1` monitors the governance contracts deployed prior to September-2018.
- `-k` monitors the `VotingToChangeKeys` contract.
- `-t` monitors the `VotingToChangeMinThreshold` contract.
- `--earliest` start monitoring from the first block in the blockchain.
- `--email` sends out email notifications to each address in the `EMAIL_RECIPIENTS` env-var.
- `--log-emails` for each governance notification generated, log the corresponding email body.
- `--limit=1` stop running `poagov` after one ballot notification has been generated.

### Logs

Logs are written to `stderr` by default; if the `--log-file` CLI flag is set,
then logs will be written to a file in the `logs.` directory. Logged
information includes: the generation of governance notifications, sending an
email successfully, failing to send an email, blocks that we have finished
monitoring, email bodies generated (using the `--log-emails` CLI option).

The following is an example command with its corresponding logs:

    $ poagov --sokol --v1 --threshold --earliest --limit=3

    Oct 10 15:18:09.863 INFO starting poagov...
    Oct 10 15:18:10.287 INFO governance notification, block_number: 525296, ballot_id: 0, ballot: Threshold
    Oct 10 15:18:10.287 INFO governance notification, block_number: 599789, ballot_id: 1, ballot: Threshold
    Oct 10 15:18:10.287 INFO governance notification, block_number: 1078816, ballot_id: 2, ballot: Threshold
    Oct 10 15:18:10.287 WARN reached notification limit, gracefully shutting down..., limit: 3

