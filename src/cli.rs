use clap::{ArgMatches, App};

pub struct Cli;

impl Cli {
    pub fn load() -> ArgMatches<'static> {
        App::new("poagov")
            .version("1.0")
            .about("A tool to monitor POA Network's blockchain for governance events.")
            .args_from_usage(
                "[network] --network [value] 'the name of the network to monitor for ballots; the values available for this option are: core, sokol, local'
                [core] --core 'monitor voting contracts deployed to the Core network (same as using --network=core)'
                [sokol] --sokol 'monitor voting contracts deployed to the Sokol test network (same as using --network=sokol)'
                [local] --local 'monitor voting contracts deployed to a locally running POA chain (same as using --network=local)'
                [rpc] --rpc [value] 'the URL for the RPC endpoint'
                [monitor] --monitor [value] 'a comma-separated list of ballot types to monitor for governance events; the available values are: keys, threshold, proxy`
                [keys] -k 'monitor the blockchain for ballots to change keys (same as --monitor=keys)'
                [threshold] -t 'monitor the chain for ballots to change the minimum threshold (same as --monitor=threshold)'
                [proxy] -p 'monitor the change for ballots to change the proxy address (same as --monitor=proxy)'
                [start_block] --start [value] 'start monitoring for governance events at this block (inclusive)'
                [tail] --tail [value] 'start monitoring for governance events for the `n` blocks prior to the last mined block in the chain'
                [earliest] --earliest 'start monitoring for goverance events starting from the first block in the chain'
                [latest] --latest 'start monitoring for goverance events starting from the most recently mined block in the chain'
                [email] --email 'send governance notifications via email'
                [push] --push 'send governance notifications via push notification'
                [block_time] --block-time [value] 'the average time it takes to mine a new block'"
            )
            .get_matches()
    }
}
