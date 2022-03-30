extern crate core;

mod account;
mod balance;
mod error;
mod processor;
mod transaction;

use crate::account::Account;
use crate::processor::{InputEvent, Processor};
use anyhow::Result;
use csv::Trim;
use rayon::prelude::*;
use std::env::args;
use std::fs::File;
use std::path::Path;

// NOTE: `tokio` is not needed for this task
fn main() -> Result<()> {
    // NOTE: `Arc` is not needed `Processor` has only one
    // field `accounts` with `DashMap` as a thread-safe structure
    let p = Processor::new();

    // obtaining input file
    let input_file = match args().collect::<Vec<String>>().get(1) {
        None => panic!("input file not provided"),
        Some(filename) => match File::open(Path::new(filename)) {
            Ok(f) => f,
            Err(e) => panic!("file not found: {}", e),
        },
    };

    // ----------------------------------------------------------------------------
    // WARNING:
    // Errors are accounted for, but not handled due to the
    // simplicity of a test project.

    csv::ReaderBuilder::new()
        .trim(Trim::All)
        .has_headers(true)
        .from_reader(input_file) // NOTE: `csv` crate automatically handles buffered reading
        .deserialize()
        .par_bridge()
        .for_each(|entry: std::result::Result<InputEvent, csv::Error>| {
            // handling transaction input
            if let Ok(InputEvent {
                typ,
                client_id,
                tx_id,
                amount,
            }) = entry
            {
                if p.handle_event(InputEvent {
                    typ,
                    client_id,
                    tx_id,
                    amount,
                })
                .is_err()
                {
                    // eprintln!("failed to handle event {}: {}", tx_id, e);
                }
            };
        });

    // dumping accounts as csv to stdout
    p.dump_accounts();

    Ok(())
}
