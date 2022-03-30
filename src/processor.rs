use crate::balance::Balance;
use crate::transaction::Event;
use crate::Account;
use anyhow::*;
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct InputEvent {
    #[serde(rename = "type")]
    pub typ: String,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub tx_id: u32,
    pub amount: Option<Decimal>,
}

#[derive(Debug)]
pub struct Processor {
    accounts: Arc<DashMap<u16, Account>>,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(Default::default()),
        }
    }

    pub fn handle_event(&self, input: InputEvent) -> Result<()> {
        let client_id = input.client_id;
        let event = Event::try_from(input)?;

        match self.accounts.entry(client_id) {
            Entry::Occupied(mut e) => e.get_mut().apply_event(event)?,
            Entry::Vacant(e) => e.insert(Account::new(client_id)).apply_event(event)?,
        };

        Ok(())
    }

    #[inline]
    #[allow(dead_code)]
    // This function is used for convenience in a testing macro
    pub fn is_account_locked(&self, client_id: u16) -> bool {
        self.accounts
            .get(&client_id)
            .map_or(false, |acc| acc.is_locked)
    }

    #[inline]
    #[allow(dead_code)]
    // This function is used for convenience in a testing macro
    pub fn get_balance(&self, client_id: u16) -> Balance {
        self.accounts
            .get(&client_id)
            .map_or(Balance::default(), |acc| acc.balance)
    }

    // no whizzbang, just print
    pub fn dump_accounts(&self) {
        println!("client,available,held,total,locked");

        self.accounts.iter().for_each(|acc| {
            println!(
                "{},{:0.4?},{:0.4?},{:0.4?},{}",
                acc.id,
                acc.balance.get_available(),
                acc.balance.held,
                acc.balance.total,
                acc.is_locked
            )
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::FromPrimitive;
    use rand::seq::SliceRandom;
    use rand::Rng;
    use rayon::prelude::*;

    #[test]
    fn test_processor_error_spam() {
        let p = Arc::new(Processor::new());

        let mut rng = rand::thread_rng();
        let types = ["deposit", "withdrawal", "dispute", "resolve", "chargeback"];

        (1..=1_000_000)
            .map(|i| InputEvent {
                typ: types.choose(&mut rng).unwrap().to_string(),
                client_id: rng.gen_range(1..10),
                tx_id: i as u32,
                amount: Decimal::from_f32(rng.gen_range(50.0..=500.0)),
            })
            .collect::<Vec<InputEvent>>()
            .into_par_iter()
            .for_each(|event| {
                let input_tx_id = event.tx_id;

                if let Err(e) = p.handle_event(event) {
                    eprintln!("failed to handle event {}: {}", input_tx_id, e);
                }
            });
    }

    macro_rules! handle_event {
        ($handle:ident, $typ:literal, $client_id:expr, $tx_id:expr, $amount:expr,
        $expected_available:expr, $expected_held:literal,
        $account_is_locked:literal, $must_fail:literal) => {
            let result = $handle.handle_event(InputEvent {
                typ: $typ.to_string(),
                client_id: $client_id,
                tx_id: $tx_id,
                amount: Some(Decimal::from_f32($amount).unwrap()),
            });

            let balance = $handle.get_balance($client_id);
            let expected_available = Decimal::from_f32($expected_available).unwrap();
            let expected_held = Decimal::from_f32($expected_held).unwrap();

            assert_eq!(result.is_err(), $must_fail);
            assert_eq!($handle.is_account_locked($client_id), $account_is_locked);
            assert_eq!(balance.get_available(), expected_available);
            assert_eq!(balance.held, expected_held);
            assert_eq!(balance.total, expected_available + expected_held);
        };
    }

    #[test]
    #[allow(clippy::bool_assert_comparison)]
    fn test_handler() {
        // ----------------------------------------------------------------------------
        // An all-in-one, simple test, for all transaction types.
        // As I understand, deposits and withdrawals should
        // still pass despite some transactions being in dispute or resolved.

        let p = Arc::new(Processor::new());

        // ----------------------------------------------------------------------------
        // dispute

        handle_event!(p, "deposit", 1, 1, 100.0, 100.0, 0.0, false, false);
        handle_event!(p, "deposit", 1, 2, 50.0, 150.0, 0.0, false, false); // disputed
        handle_event!(p, "withdrawal", 1, 3, 50.0, 100.0, 0.0, false, false);
        handle_event!(p, "dispute", 1, 2, 0.0, 50.0, 50.0, false, false);
        handle_event!(p, "deposit", 1, 4, 10.0, 60.0, 50.0, false, false);
        handle_event!(p, "withdrawal", 1, 5, 10.0, 50.0, 50.0, false, false);
        handle_event!(p, "withdrawal", 1, 6, 70.0, 50.0, 50.0, false, true); // must fail

        // ----------------------------------------------------------------------------
        // resolution

        handle_event!(p, "deposit", 2, 1, 100.0, 100.0, 0.0, false, false);
        handle_event!(p, "deposit", 2, 2, 50.0, 150.0, 0.0, false, false); // disputed
        handle_event!(p, "withdrawal", 2, 3, 50.0, 100.0, 0.0, false, false);
        handle_event!(p, "dispute", 2, 2, 0.0, 50.0, 50.0, false, false);
        handle_event!(p, "deposit", 2, 4, 10.0, 60.0, 50.0, false, false);
        handle_event!(p, "withdrawal", 2, 5, 10.0, 50.0, 50.0, false, false);
        handle_event!(p, "resolve", 2, 2, 0.0, 100.0, 0.0, false, false);
        handle_event!(p, "dispute", 2, 2, 0.0, 100.0, 0.0, false, true); // recurring dispute must fail

        // ----------------------------------------------------------------------------
        // chargeback and account locking

        handle_event!(p, "deposit", 3, 1, 100.0, 100.0, 0.0, false, false);
        handle_event!(p, "deposit", 3, 2, 50.0, 150.0, 0.0, false, false); // disputed, charged back and account locked
        handle_event!(p, "withdrawal", 3, 3, 50.0, 100.0, 0.0, false, false);
        handle_event!(p, "dispute", 3, 2, 0.0, 50.0, 50.0, false, false);
        handle_event!(p, "deposit", 3, 4, 10.0, 60.0, 50.0, false, false);
        handle_event!(p, "withdrawal", 3, 5, 10.0, 50.0, 50.0, false, false);
        handle_event!(p, "chargeback", 3, 2, 0.0, 50.0, 0.0, true, false);
        handle_event!(p, "dispute", 3, 2, 0.0, 50.0, 0.0, true, true); // dispute after chargeback must fail
        handle_event!(p, "deposit", 3, 6, 50.0, 50.0, 0.0, true, true); // deposit after chargeback must fail
    }
}
