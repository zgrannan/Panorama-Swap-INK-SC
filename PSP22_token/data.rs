use crate::PSP22Error;
use ink::{
    prelude::{string::String, vec, vec::Vec},
    storage::Mapping,
};

type AccountId = u32;

/// A class implementing the internal logic of a PSP22 token.
///
/// Holds the state of all account balances and allowances.
/// Each method of this class corresponds to one type of transaction
/// as defined in the PSP22 standard.
///
/// Since this code is outside of `ink::contract` macro, the caller's
/// address cannot be obtained automatically. Because of that, all
/// the methods that need to know the caller require an additional argument
/// (compared to transactions defined by the PSP22 standard or the PSP22 trait).
///
/// `lib.rs` contains an example implementation of a smart contract using this class.
pub struct PSP22Data {
    total_supply: u128,
    balances: Mapping<AccountId, u128>,
    allowances: Mapping<(AccountId, AccountId), u128>,
}

impl PSP22Data {
    /// Creates a token with `supply` balance, initially held by the `creator` account.
    pub fn new(supply: u128, creator: AccountId) -> PSP22Data {
        let mut data = PSP22Data {
            total_supply: supply,
            balances: Default::default(),
            allowances: Default::default(),
        };
        data.balances.insert(creator, &supply);
        data
    }

    pub fn total_supply(&self) -> u128 {
        self.total_supply
    }

    pub fn balance_of(&self, owner: AccountId) -> u128 {
        self.balances.get(owner).unwrap_or_default()
    }

    pub fn allowance(&self, owner: AccountId, spender: AccountId) -> u128 {
        self.allowances.get((owner, spender)).unwrap_or_default()
    }

    /// Transfers `value` tokens from `caller` to `to`.
    pub fn transfer(
        &mut self,
        caller: AccountId,
        to: AccountId,
        value: u128,
    ) -> Result<(), PSP22Error> {
        if caller == to || value == 0 {
            return Ok(());
        }
        let from_balance = self.balance_of(caller);
        if from_balance < value {
            return Err(PSP22Error::InsufficientBalance);
        }

        self.balances
            .insert(caller, &(from_balance.saturating_sub(value)));
        let to_balance = self.balance_of(to);
        self.balances
            .insert(to, &(to_balance.saturating_add(value)));
        Ok(())
    }

    /// Transfers `value` tokens from `from` to `to`, but using the allowance
    /// granted by `from` to `caller`.
    pub fn transfer_from(
        &mut self,
        caller: AccountId,
        from: AccountId,
        to: AccountId,
        value: u128,
    ) -> Result<(), PSP22Error> {
        if from == to || value == 0 {
            return Ok(());
        }
        if caller == from {
            return self.transfer(caller, to, value);
        }

        let allowance = self.allowance(from, caller);
        if allowance < value {
            return Err(PSP22Error::InsufficientAllowance);
        }
        self.allowances
            .insert((from, caller), &(allowance.saturating_sub(value)));

        self.transfer(from, to, value)
    }

    /// Sets a new `value` for allowance granted by `owner` to `spender`.
    /// Overwrites the previously granted value.
    pub fn approve(
        &mut self,
        owner: AccountId,
        spender: AccountId,
        value: u128,
    ) -> Result<(), PSP22Error> {
        if owner == spender {
            return Ok(());
        }
        self.allowances.insert((owner, spender), &value);
        Ok(())
    }

    /// Increases the allowance granted by `owner` to `spender` by `delta_value`.
    pub fn increase_allowance(
        &mut self,
        owner: AccountId,
        spender: AccountId,
        delta_value: u128,
    ) -> Result<(), PSP22Error> {
        if owner == spender || delta_value == 0 {
            return Ok(());
        }
        let allowance = self.allowance(owner, spender).saturating_add(delta_value);
        self.allowances.insert((owner, spender), &allowance);
        Ok(())
    }

    /// Decreases the allowance granted by `owner` to `spender` by `delta_value`.
    pub fn decrease_allowance(
        &mut self,
        owner: AccountId,
        spender: AccountId,
        delta_value: u128,
    ) -> Result<(), PSP22Error> {
        if owner == spender || delta_value == 0 {
            return Ok(());
        }
        let allowance = self.allowance(owner, spender);
        if allowance < delta_value {
            return Err(PSP22Error::InsufficientAllowance);
        }
        let new_allowance = allowance.saturating_sub(delta_value);
        self.allowances.insert((owner, spender), &new_allowance);
        Ok(())
    }

    /// Mints `value` of new tokens to `to` account.
    pub fn mint(&mut self, to: AccountId, value: u128) -> Result<(), PSP22Error> {
        if value == 0 {
            return Ok(());
        }
        let new_supply = self.total_supply.checked_add(value)
            .ok_or_else(|| PSP22Error::Custom(String::from("Max PSP22 supply exceeded. Max supply limited to 2^128-1.")))?;
        self.total_supply = new_supply;
        let new_balance = self.balance_of(to).saturating_add(value);
        self.balances.insert(to, &new_balance);
        Ok(())
    }

    /// Burns `value` tokens from `from` account.
    pub fn burn(&mut self, from: AccountId, value: u128) -> Result<(), PSP22Error> {
        if value == 0 {
            return Ok(());
        }
        let balance = self.balance_of(from);
        if balance < value {
            return Err(PSP22Error::InsufficientBalance);
        }
        self.balances.insert(from, &(balance.saturating_sub(value)));
        self.total_supply = self.total_supply.saturating_sub(value);
        Ok(())
    }
}
