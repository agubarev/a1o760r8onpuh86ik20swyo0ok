use crate::InputEvent;
use anyhow::bail;
use num_traits::Zero;
use rust_decimal::Decimal;

#[derive(Debug)]
pub enum Status {
    Normal,
    Disputed,
    Resolved,
    ChargedBack,
}

#[derive(Debug)]
pub enum Kind {
    Deposit,
    Withdrawal,
}

#[derive(Debug)]
pub enum Event {
    Deposit {
        tx_id: u32,
        client_id: u16,
        amount: Decimal,
    },
    Withdrawal {
        tx_id: u32,
        client_id: u16,
        amount: Decimal,
    },
    Dispute {
        client_id: u16,
        disputed_tx_id: u32,
    },
    Resolve {
        client_id: u16,
        disputed_tx_id: u32,
    },
    Chargeback {
        client_id: u16,
        disputed_tx_id: u32,
    },
}

#[derive(Debug)]
pub struct Entry {
    pub tx: Event,
    pub kind: Kind,
    pub status: Status,
}

impl Entry {
    pub fn new(kind: Kind, tx: Event) -> Self {
        Self {
            tx,
            kind,
            status: Status::Normal,
        }
    }
}

impl TryFrom<InputEvent> for Event {
    type Error = anyhow::Error;

    fn try_from(entry: InputEvent) -> Result<Self, Self::Error> {
        Ok(match entry.typ.as_str() {
            "deposit" => Event::Deposit {
                tx_id: entry.tx_id,
                client_id: entry.client_id,
                amount: entry.amount.unwrap_or_else(Decimal::zero),
            },
            "withdrawal" => Event::Withdrawal {
                tx_id: entry.tx_id,
                client_id: entry.client_id,
                amount: entry.amount.unwrap_or_else(Decimal::zero),
            },
            "dispute" => Event::Dispute {
                disputed_tx_id: entry.tx_id,
                client_id: entry.client_id,
            },
            "resolve" => Event::Resolve {
                disputed_tx_id: entry.tx_id,
                client_id: entry.client_id,
            },
            "chargeback" => Event::Chargeback {
                disputed_tx_id: entry.tx_id,
                client_id: entry.client_id,
            },
            _ => bail!("unrecognized input entry type: {}", entry.typ),
        })
    }
}
