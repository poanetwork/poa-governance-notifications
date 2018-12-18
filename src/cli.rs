// Some of `Cli`'s methods are not currently being used.
#![allow(dead_code)]

use clap::{App, ArgMatches};

pub fn parse_cli() -> Cli {
    let cli_args = App::new("poagov")
        .version("1.0.0")
        .about("Monitors a POA Network blockchain for governance events.")
        .args_from_usage(
            "[core] --core 'Monitors POA Network's Core Network for governance ballots'
            [sokol] --sokol 'Monitors POA Network's Sokol network for governance ballots'
            [xdai] --xdai 'Monitors the xDai Network for governance ballots'
            [keys] -k --keys 'Monitors the blockchain for ballots to change keys'
            [threshold] -t --threshold 'Monitors the blockchain for ballots to change the minimum threshold'
            [proxy] -p --proxy 'Monitors the blockchain for ballots to change the proxy address'
            [emission] -e --emission 'Monitors the blockchain for ballots to manage emission funds'
            [v1] --v1 'Monitors the v1 governance contracts'
            [v2] --v2 '[default] Monitors the v2 governance contracts, if no contract version CLI argument is given by the user, we set this CLI flag'
            [earliest] --earliest 'Monitor for governance events starting at the blockchain's first block'
            [latest] --latest 'Monitor for governance events starting at the blockchain's most recently mined block'
            [start_block] --start [value] 'Start monitoring for governance events at this block (inclusive)'
            [tail] --tail [value] 'Start monitoring for governance events for the `n` blocks prior to the last mined block'
            [email] --email 'Enables email notifications (SMTP configuration options must be set in your `.env` file)'
            [block_time] --block-time [value] 'The average number of seconds it takes to mine a new block'
            [notification_limit] -n --limit [value] 'Stops `poagov` after this many notifications have been generated (this option can be useful when testing `poagov`)'
            [log_emails] --log-emails 'Logs the full email body for each notification generated, this option does not require the `--email` flag to be set'
            [log_to_file] --log-file 'Logs are written to files in the ./logs directory, logs are rotated chronologically across 3 files, each file has a max size of 8MB'"
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

    pub fn xdai(&self) -> bool {
        self.0.is_present("xdai")
    }

    pub fn one_network_specified(&self) -> bool {
        match (self.core(), self.sokol(), self.xdai()) {
            (true, false, false) => true,
            (false, true, false) => true,
            (false, false, true) => true,
            _ => false,
        }
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

    pub fn no_contracts_specified(&self) -> bool {
        !self.keys() && !self.threshold() && !self.proxy() && !self.emission()
    }

    pub fn v1(&self) -> bool {
        self.0.is_present("v1")
    }

    pub fn v2(&self) -> bool {
        self.0.is_present("v2")
    }

    pub fn multiple_versions_specified(&self) -> bool {
        self.v1() && self.v2()
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

    pub fn one_start_block_was_specified(&self) -> bool {
        match (self.earliest(), self.latest(), self.start_block().is_some(), self.tail().is_some()) {
            (true, false, false, false) => true,
            (false, true, false, false) => true,
            (false, false, true, false) => true,
            (false, false, false, true) => true,
            _ => false,
        }
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
