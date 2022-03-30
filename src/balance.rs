use crate::error::BalanceError;
use anyhow::{bail, Result};
use num_traits::Zero;
use rust_decimal::Decimal;

#[derive(Debug, Copy, Clone, Default)]
pub struct Balance {
    pub total: Decimal,
    pub held: Decimal,
}

impl Balance {
    pub fn new() -> Self {
        Self {
            total: Decimal::zero(),
            held: Decimal::zero(),
        }
    }

    #[inline]
    pub fn get_available(&self) -> Decimal {
        self.total - self.held
    }

    #[inline]
    pub fn has_enough_available(&self, amount: Decimal) -> bool {
        self.get_available() >= amount
    }

    #[inline]
    pub fn has_enough_held(&self, amount: Decimal) -> bool {
        self.held >= amount
    }

    #[inline]
    pub fn credit(&mut self, amount: Decimal) {
        self.total += amount;
    }

    #[inline]
    pub fn debit(&mut self, amount: Decimal) -> Result<()> {
        if !self.has_enough_available(amount) {
            bail!(BalanceError::InsufficientAvailableFunds {
                available: self.get_available(),
                required: amount,
            });
        }

        self.total -= amount;

        Ok(())
    }

    #[inline]
    pub fn hold(&mut self, amount: Decimal) -> Result<()> {
        if !self.has_enough_available(amount) {
            bail!(BalanceError::InsufficientAvailableFunds {
                available: self.get_available(),
                required: amount,
            });
        }

        self.held += amount;

        Ok(())
    }

    #[inline]
    pub fn burn_held(&mut self, amount: Decimal) -> Result<()> {
        if !self.has_enough_held(amount) {
            bail!(BalanceError::InsufficientHeldFunds {
                held: self.held,
                required: amount,
            });
        }

        self.held -= amount;
        self.total -= amount;

        Ok(())
    }

    #[inline]
    pub fn release_held(&mut self, amount: Decimal) -> Result<()> {
        if !self.has_enough_held(amount) {
            bail!(BalanceError::InsufficientHeldFunds {
                held: self.held,
                required: amount,
            });
        }

        self.held -= amount;

        Ok(())
    }
}
