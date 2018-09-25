[![Build Status](https://travis-ci.org/poanetwork/poa-governance-notifications.svg?branch=master)](https://travis-ci.org/poanetwork/poa-governance-notifications) 

# About

A tool to monitor a POA Network blockchain for
[governance events](https://github.com/poanetwork/wiki/wiki/Governance-Overview).

The `poagov` command line tool is distributed as a binary for Linux and
OSX; it can also be built from source for both platforms.

# Installing the `poagov` Binary

*Note:* the `poagov` binary requires libssl to be installed prior to
usage, if you do not have libssl installed, go to the "Requires libssl"
section in this README to find out how to download it.  

On Debian/Ubuntu:

    $ curl -OL https://github.com/poanetwork/poa-governance-notifications/releases/download/v1.0.0/poagov-1.0.0-linux-x86_64.tar.gz
    $ tar -xvzf poagov-1..0-linux-x86_64.tar.gz
    $ rm poagov-1.0.0-linux-x86_64.tar.gz
    $ cd poagov
    $ chmod +x poagov
    $ ./poagov --help

On OSX:

    $ curl -OL https://github.com/poanetwork/poa-governance-notifications/releases/download/v1.0.0/poagov-1.0.0-osx-x86_64.tar.gz
    $ tar -xvzf poagov-1.0.0-osx-x86_64.tar.gz
    $ rm poagov-1.0.0-osx-x86_64.tar.gz
    $ cd poagov
    $ chmod +x poagov
    $ ./poagov --help

Make sure you have an `.env` file in the same directory as the `poagov`
binary; see the section "Setting up the `.env` File" for more
information.

# Building `poagov` from Source

To build the `poagov` CLI tool, run the following:

    $ git clone https://github.com/poanetwork/poa-governance-notifications.git
    $ cd poa-governance-notifications
    $ cargo build --release

`poagov` can be built using Rust 1.29 stable and requires `libssl` to be
installed; see the following "Requires libssl" section for more information.

You can run `poagov`'s tests via the following command (make sure to copy
`sample.env` into `.env` before testing):

    $ cargo test

### Requires `libssl`

SMTP over TLS requires that you have a native TLS library installed on your
machine, the preferred library for Linux and OSX is OpenSSL >= 1.0.1,
otherwise known as `libssl` (you will need more than just the OpenSSL
binary that you may or may not already have installed at
`/usr/bin/openssl`).

If running `cargo build [--release]` panics with an error like:

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
          cargo build [--release]

There is a known issue regarding the `openssl-sys` crate not being able to
find `libssl` installed with Homebrew on OSX; more information can be found on
[Stack Overflow](https://stackoverflow.com/questions/34612395/openssl-crate-fails-compilation-on-mac-os-x-10-11/34615626#34615626).
The above solution comes from the linked Stack Overflow thread.

More information on common issues encountered while installing the
`openssl` Rust crate can be found [here](https://crates.io/crates/openssl).

# Usage

Once you have built or downloaded `poagov`, you can print out the CLI usage by
running:

    $ poagov --help
    # If built from source run:
    # $ ./target/{debug, release}/poagov --help

    poagov 1.0.0
    Monitors a POA Network blockchain for governance events.

    USAGE:
        poagov [FLAGS] [OPTIONS]

    FLAGS:
        --core             monitor voting contracts deployed to the Core network
	--earliest         begin monitoring for governance events starting at the first block in the blockchain
	--email            enables email notifications (SMTP configurations must be set in your `.env` file)
	-e, --emission     monitors the blockchain for ballots to manage emission funds
	-h, --help         Prints help information
	-k, --keys         monitors the blockchain for ballots to change keys
	--latest           begin monitoring for governance events starting at the last block mined
	-p, --proxy        monitors the blockchain for ballots to change the proxy address
	--sokol            monitor voting contracts deployed to the Sokol network
	-t, --threshold    monitors the blockchain for ballots to change the minimum threshold
	--v1               monitors the v1 voting contracts
	--v2               monitors the v2 voting contracts
	-V, --version      Prints version information
	--verbose          prints the full notification email's body when logging

    OPTIONS:
	--block-time <value>    the average number of seconds it takes to mine a new block
        -n, --limit <value>     shutdown `poagov` after this many notifications have been generated
	--start <value>         start monitoring for governance events at this block (inclusive)
	--tail <value>          start monitoring for governance events for the `n` blocks prior to the last block minedV

### Required Arguments

Each time you run `poagov`, four CLI arguments are required:

1. The chain that you want to monitor. Uou must specify one and only one of
the following arguments: `--core` or `--sokol`.
2. The hardfork version. You must specify one of the following: `--v1` or `--v2`.
    - `--v1` indicates that you want to monitor for governance events prior to
    the Sokol and Core hardforks that will occur in September-2018 and
    November-2018 respectively.
    - `--v2` indicates that you want to monitor for governance events that
    occured after the above hardfork dates.
    - More information regarding the planned hardforks for the Sokol and Core
    chains in September and November 2018 can be found
    [here](https://medium.com/poa-network/poa-network-news-and-updates-36-2e6e00550c15).
3. The ballots that you want to monitor for governance events. You must specify
one or more of the following arguments: `-k`/`--keys`, `-t`/`--threshold`,
`-p`/`--proxy`, and/or `-e`/`--emission`.
    - Note that the `VotingToManageEmissionFunds.sol` contract (i.e. the
    `--emission` option) is not available in `--v1`.
4. The point in the chain for where to start monitoring. You must specify one
and only one of the following: `--earliest`, `--latest`, `--start=<value>`, or
`--tail=<value>`.

### Optional Arguments

Providing the `--email` flag will enable governance notification emails. To use
this option, you must first configure SMTP in your `.env`.

Providing the `--block-time=<value>` will set how often `poagov` will query the
blockchain for new governance events. Defaults to 30 seconds.

Providing the `--verbose` flag will print the full text for a notification
email to stderr when governance events are found. When this option is set,
email text will be logged regardless of whether or not the `--email` flag is
set.

# Setting up the `.env` File

When the `poagov` CLI tool is run, the process' environment variables are
loaded via an `.env` file. Before running `poagove` copy `sample.env` into
`.env`:

    $ cp sample.env .env

This will enable `poagov's` default configuration. Before enabling email
notifications, you must add the required SMTP configuration values to your
`.env` file.

# Setting up Email Notifications

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

    $ poagov --sokol --earliest -kt --email --verbose

- `--sokol` is used to monitor contracts deployed to POA's test network.
- `--earliest` starts monitoring from the first block in the blockchain.
- `-k` get notifications for ballots to change keys.
- `-t` get notifications for ballots to change the min threshold.
- `--email` sends out email notifications to each address in the
`EMAIL_RECIPIENTS` env-var.
- `--verbose` writes each governance notification email to stderr.

Press [ctrl-c] to exit `poagov`.

# Logs

Logs are output to stderr. Logs include: governance notifications, email
successes/failures, and blocks that have been successfully monitored for
governance events. The following is an example command with its corresponding
logs:

    $ poagov --sokol --v1 --threshold --earliest

    Sep 25 13:43:16.712 INFO governance notification, block_number: 525296, ballot_id: 0, ballot: Threshold
    Sep 25 13:43:16.712 INFO governance notification, block_number: 599789, ballot_id: 1, ballot: Threshold
    Sep 25 13:43:16.712 INFO governance notification, block_number: 1078816, ballot_id: 2, ballot: Threshold
    Sep 25 13:43:16.712 INFO finished checking blocks, block_range: Number(0)...Number(4729306)
    Sep 25 13:43:46.761 INFO finished checking blocks, block_range: Number(4729307)...Number(4729312)
    Sep 25 13:43:48.503 WARN recieved ctrl-c signal, gracefully shutting down...

