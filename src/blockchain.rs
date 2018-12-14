use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use web3::types::BlockNumber;

use crate::client::RpcClient;
use crate::config::{Config, StartBlock};
use crate::error::{Error, Result};

/// Represents the reason why the sleep cycle in `fn sleep_or_ctrlc()` ended.
#[derive(PartialEq)]
enum SleepExit {
    CtrlC,
    FinishedSleeping,
}

/// Sleeps for `n_secs` number of seconds or returns early if the user shuts down `poagov` using
/// ctrl-c.
///
/// Returns  `SleepExit::CtrlC` if the user hit ctrl-c while this function was was sleeping or
/// returns `SleepExit::FinishedSleeping` if the function was able to sleep for the entire `n_secs`
/// duration.
fn sleep_or_ctrlc(n_secs: u64, running: Arc<AtomicBool>) -> SleepExit {
    // This `AtomicBool` will become `true` when we have slept for `n_secs`.
    let done_sleeping = Arc::new(AtomicBool::new(false));
    {
        let done_sleeping = done_sleeping.clone();
        let _handle = thread::spawn(move || {
            thread::sleep(Duration::from_secs(n_secs));
            done_sleeping.store(true, Ordering::SeqCst);
        });
    }
    while !done_sleeping.load(Ordering::SeqCst) {
        if !running.load(Ordering::SeqCst) {
            return SleepExit::CtrlC;
        }
    }
    SleepExit::FinishedSleeping
}

/// A type that we use to iterate over the blocks in a blockchain in discrete block-windows (each
/// "block-window" is an inclusively bounded range of block numbers).
pub struct BlockchainIter<'a> {
    client: &'a RpcClient,
    start_block: u64,
    stop_block: u64,
    on_first_iteration: bool,
    block_time: u64,
    running: Arc<AtomicBool>,
}

impl<'a> BlockchainIter<'a> {
    /// Creates a new `BlockchainIter`.
    ///
    /// # Errors
    ///
    /// Return an error if the HTTP-RPC server cannot be reached or if the response from the RPC
    /// server  cannot be parsed.
    ///
    /// Returns an `Error::StartBlockExceedsLastBlockMined` if the `start_block` that the user
    /// passed in via a CLI argument is in the future (i.e. is greater than the block number of the
    /// most recently mined block).
    pub fn new(client: &'a RpcClient, config: &Config, running: Arc<AtomicBool>) -> Result<Self> {
        let last_mined_block = client.get_last_mined_block_number()?;
        let start_block = match config.start_block {
            StartBlock::Earliest => 0,
            StartBlock::Latest => last_mined_block,
            StartBlock::Number(block_number) => block_number,
            StartBlock::Tail(tail) => last_mined_block - tail,
        };
        if start_block > last_mined_block {
            return Err(Error::StartBlockExceedsLastBlockMined {
                start_block,
                last_mined_block,
            });
        }
        Ok(BlockchainIter {
            client,
            start_block,
            stop_block: last_mined_block,
            on_first_iteration: true,
            block_time: config.block_time,
            running,
        })
    }
}

impl<'a> Iterator for BlockchainIter<'a> {
    type Item = Result<(BlockNumber, BlockNumber)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.on_first_iteration {
            self.on_first_iteration = false;
        } else {
            self.start_block = self.stop_block + 1;
            while self.start_block >= self.stop_block {
                if sleep_or_ctrlc(self.block_time, self.running.clone()) == SleepExit::CtrlC {
                    return None;
                }
                self.stop_block = match self.client.get_last_mined_block_number() {
                    Ok(last_mined) => last_mined,
                    Err(e) => return Some(Err(e)),
                };
            }
        };
        if self.running.load(Ordering::SeqCst) {
            let range = (self.start_block.into(), self.stop_block.into());
            Some(Ok(range))
        } else {
            None
        }
    }
}
