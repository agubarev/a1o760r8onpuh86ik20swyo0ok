use crate::balance::Balance;
use crate::error::{AccountError, TransactionError};
use crate::transaction::{Entry, Event, Kind, Status};
use anyhow::{bail, Result};
use dashmap::DashMap;
use rust_decimal::Decimal;

#[derive(Debug)]
pub struct Account {
    pub id: u16,
    pub is_locked: bool,
    pub balance: Balance,
    txs: DashMap<u32, Entry>,
}

// WARNING: not sure how to handle withdrawal disputes without
// additional reimbursement information, nor see the worth bothering for a fun toy
impl Account {
    pub fn new(id: u16) -> Self {
        Self {
            id,
            balance: Balance::new(),
            is_locked: false,
            txs: Default::default(),
        }
    }

    pub fn apply_event(&mut self, tx: Event) -> Result<()> {
        if self.is_locked {
            bail!(AccountError::AccountLocked);
        }

        match tx {
            Event::Deposit { amount, tx_id, .. } => {
                self.deposit(amount);
                self.txs.insert(tx_id, Entry::new(Kind::Deposit, tx));
            }
            Event::Withdrawal { amount, tx_id, .. } => {
                self.withdraw(amount)?;
                self.txs.insert(tx_id, Entry::new(Kind::Withdrawal, tx));
            }
            Event::Dispute { disputed_tx_id, .. } => self.dispute(disputed_tx_id)?,
            Event::Resolve { disputed_tx_id, .. } => self.resolve(disputed_tx_id)?,
            Event::Chargeback { disputed_tx_id, .. } => self.chargeback(disputed_tx_id)?,
        };

        Ok(())
    }

    fn deposit(&mut self, amount: Decimal) {
        self.balance.credit(amount);
    }

    fn withdraw(&mut self, amount: Decimal) -> Result<()> {
        self.balance.debit(amount)
    }

    fn dispute(&mut self, tx_id: u32) -> Result<()> {
        if let Some(mut entry) = self.txs.get_mut(&tx_id) {
            match entry.tx {
                Event::Deposit { amount, .. } => match entry.status {
                    Status::Normal => {
                        // withholding a disputed amount
                        self.balance.hold(amount)?;

                        // marking entry as resolved
                        entry.status = Status::Disputed;
                    }

                    Status::Disputed => bail!(TransactionError::AlreadyDisputed { tx_id }),
                    Status::Resolved => bail!(TransactionError::AlreadyResolved { tx_id }),
                    Status::ChargedBack => bail!(TransactionError::AlreadyChargedBack { tx_id }),
                },
                _ => {
                    bail!(TransactionError::InvalidResolutionTarget {
                        tx_id,
                        expected_kind: Kind::Deposit,
                    });
                }
            }
        } else {
            bail!(TransactionError::NotFound { tx_id });
        }

        Ok(())
    }

    fn resolve(&mut self, tx_id: u32) -> Result<()> {
        if let Some(mut entry) = self.txs.get_mut(&tx_id) {
            match entry.tx {
                Event::Deposit { amount, .. } => match entry.status {
                    Status::Disputed => {
                        // releasing previously held amount of a disputed transaction
                        self.balance.release_held(amount)?;

                        // marking entry as resolved
                        entry.status = Status::Resolved;
                    }

                    Status::Normal => bail!(TransactionError::NotDisputed { tx_id }),
                    Status::Resolved => bail!(TransactionError::AlreadyResolved { tx_id }),
                    Status::ChargedBack => bail!(TransactionError::AlreadyChargedBack { tx_id }),
                },
                _ => {
                    bail!(TransactionError::InvalidResolutionTarget {
                        tx_id,
                        expected_kind: Kind::Deposit,
                    });
                }
            }
        } else {
            bail!(TransactionError::NotFound { tx_id });
        }

        Ok(())
    }

    fn chargeback(&mut self, tx_id: u32) -> Result<()> {
        if let Some(mut entry) = self.txs.get_mut(&tx_id) {
            match entry.tx {
                Event::Deposit { amount, .. } => match entry.status {
                    Status::Disputed => {
                        // releasing previously held amount of a disputed transaction
                        self.balance.burn_held(amount)?;

                        // marking entry as resolved
                        entry.status = Status::ChargedBack;

                        // locking the account
                        self.is_locked = true;
                    }

                    Status::Normal => bail!(TransactionError::NotDisputed { tx_id }),
                    Status::Resolved => bail!(TransactionError::AlreadyResolved { tx_id }),
                    Status::ChargedBack => bail!(TransactionError::AlreadyChargedBack { tx_id }),
                },
                _ => {
                    bail!(TransactionError::InvalidResolutionTarget {
                        tx_id,
                        expected_kind: Kind::Deposit,
                    });
                }
            }
        } else {
            bail!(TransactionError::NotFound { tx_id });
        }

        Ok(())
    }
}
