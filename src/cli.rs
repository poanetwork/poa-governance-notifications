// Some of `Cli`'s methods are not currently being used.
#![allow(dead_code)]

use clap::{ArgMatches, App};

pub fn parse_cli() -> Cli {
    let cli_args = App::new("poagov")
        .version("1.0.0")
        .about("Monitors a POA Network blockchain for governance events.")
        .args_from_usage(
            "[core] --core 'monitor voting contracts deployed to the Core network'
            [sokol] --sokol 'monitor voting contracts deployed to the Sokol network'
            [keys] -k --keys 'monitors the blockchain for ballots to change keys'
            [threshold] -t --threshold 'monitors the blockchain for ballots to change the minimum threshold'
            [proxy] -p --proxy 'monitors the blockchain for ballots to change the proxy address'
            [emission] -e --emission 'monitors the blockchain for ballots to manage emission funds'
            [v1] --v1 'monitors the v1 voting contracts'
            [v2] --v2 'monitors the v2 voting contracts'
            [earliest] --earliest 'begin monitoring for governance events starting at the first block in the blockchain'
            [latest] --latest 'begin monitoring for governance events starting at the last block mined'
            [start_block] --start [value] 'start monitoring for governance events at this block (inclusive)'
            [tail] --tail [value] 'start monitoring for governance events for the `n` blocks prior to the last block mined'
            [email] --email 'enables email notifications (SMTP configurations must be set in your `.env` file)'
            [block_time] --block-time [value] 'the average number of seconds it takes to mine a new block'
            [notification_limit] -n --limit [value] 'shutdown `poagov` after this many notifications have been generated'
            [log_emails] --log-emails 'logs each notification's email body; does not require the --email flag to be set'
            [log_to_file] --log-file 'logs are written to files in the ./logs directory, logs are rotated chronologically across 3 files, each file has a max size of 8MB'"
        ).get_matches();

    Cli(cli_args)
}

#[derive(Debug)]
pub struct Cli(ArgMatches<'static>);

impl Cli {
    pub fn core(&self) -> bool {
        self.0.is_present("core")
    }

    pub fn sokol(&self) -> bool {
        self.0.is_present("sokol")
    }

    pub fn keys(&self) -> bool {
        self.0.is_present("keys")
    }

    pub fn threshold(&self) -> bool {
        self.0.is_present("threshold")
    }

    pub fn proxy(&self) -> bool {
        self.0.is_present("proxy")
    }

    pub fn emission(&self) -> bool {
        self.0.is_present("emission")
    }

    pub fn v1(&self) -> bool {
        self.0.is_present("v1")
    }

    pub fn v2(&self) -> bool {
        self.0.is_present("v2")
    }

    pub fn earliest(&self) -> bool {
        self.0.is_present("earliest")
    }

    pub fn latest(&self) -> bool {
        self.0.is_present("latest")
    }

    pub fn start_block(&self) -> Option<&str> {
        self.0.value_of("start_block")
    }

    pub fn tail(&self) -> Option<&str> {
        self.0.value_of("tail")
    }

    pub fn multiple_start_blocks_specified(&self) -> bool {
        let mut count = 0;
        if self.earliest() {
            count += 1;
        }
        if self.latest() {
            count += 1;
        }
        if self.start_block().is_some() {
            count += 1;
        }
        if self.tail().is_some() {
            count += 1;
        }
        count != 1
    }

    pub fn email(&self) -> bool {
        self.0.is_present("email")
    }

    pub fn block_time(&self) -> Option<&str> {
        self.0.value_of("block_time")
    }

    pub fn notification_limit(&self) -> Option<&str> {
        self.0.value_of("notification_limit")
    }

    pub fn log_emails(&self) -> bool {
        self.0.is_present("log_emails")
    }

    pub fn log_to_file(&self) -> bool {
        self.0.is_present("log_to_file")
    }
}
