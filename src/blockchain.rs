use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use web3::types::BlockNumber;

use client::RpcClient;
use config::{Config, StartBlock};
use error::{Error, Result};

fn sleep_or_ctrlc(n_secs: u64, running: Arc<AtomicBool>) -> Option<()> {
    let done_sleeping = Arc::new(AtomicBool::new(false));
    {
        let done_sleeping = done_sleeping.clone();
        let _handle = thread::spawn(move || {
            thread::sleep(Duration::from_secs(n_secs));
            done_sleeping.store(true, Ordering::SeqCst);
        });
    }
    loop {
        if !running.load(Ordering::SeqCst) {
            return None;
        }
        if done_sleeping.load(Ordering::SeqCst) {
            return Some(());
        }
    }
}

pub struct BlockchainIter<'a> {
    client: &'a RpcClient,
    start_block: u64,
    stop_block: u64,
    on_first_iteration: bool,
    block_time: u64,
    running: Arc<AtomicBool>,
}

impl<'a> BlockchainIter<'a> {
    pub fn new(client: &'a RpcClient, config: &Config, running: Arc<AtomicBool>) -> Result<Self> {
        let last_mined_block = client.get_last_mined_block_number()?;
        let start_block = match config.start_block {
            StartBlock::Earliest => 0,
            StartBlock::Latest => last_mined_block,
            StartBlock::Number(block_number) => block_number,
            StartBlock::Tail(tail) => last_mined_block - tail,
        };
        if start_block > last_mined_block {
            return Err(Error::StartBlockExceedsLastBlockMined { start_block, last_mined_block });
        }
        let bc_iter = BlockchainIter {
            client,
            start_block,
            stop_block: last_mined_block,
            on_first_iteration: true,
            block_time: config.block_time,
            running,
        };
        Ok(bc_iter)
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
                sleep_or_ctrlc(self.block_time, self.running.clone())?;
                match self.client.get_last_mined_block_number() {
                    Ok(last_mined) => self.stop_block = last_mined,
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
