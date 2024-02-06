use crate::PSP22Error;
use ink::prelude::{string::String, vec, vec::Vec};
use prusti_contracts::*;

type AccountId = u32;

struct Mapping<K, V>(u32, std::marker::PhantomData<(K, V)>);

impl<K: Copy, V: Copy> Mapping<K, V> {
    #[trusted]
    #[ensures(forall(|k : K| result.get(k) === None))]
    #[ensures(forall(|k : K| !result.get(k).is_some()))]
    fn new() -> Self {
        unimplemented!()
    }
}

impl<T: Copy, U: Copy> Mapping<T, U> {
    #[trusted]
    #[ensures(self.get(key) === Some(value))]
    #[ensures(self.get(key).is_some())]
    #[ensures(self.get(key).unwrap() === value)]
    #[ensures(forall(|k : T| k !== key ==> self.get(k) === old(self.get(k))))]
    fn insert(&mut self, key: T, value: U) {
        unimplemented!()
    }

    #[trusted]
    #[ensures(matches!(self.get(key), None))]
    #[ensures(forall(|k : T| k !== key ==> self.get(k) === old(self.get(k))))]
    fn remove(&mut self, key: T) {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    fn get(&self, key: T) -> Option<U> {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    #[ensures(result == matches!(self.get(key), Some(_)))]
    fn contains(&self, key: T) -> bool {
        unimplemented!()
    }
}

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
///
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
            balances: Mapping::new(),
            allowances: Mapping::new(),
        };
        data.balances.insert(creator, supply);
        data
    }

    #[pure]
    pub fn total_supply(&self) -> u128 {
        self.total_supply
    }

    #[pure]
    #[ensures(matches!(self.balances.get(owner), None) ==> result == 0)]
    pub fn balance_of(&self, owner: AccountId) -> u128 {
        match self.balances.get(owner) {
            Some(balance) => balance,
            None => 0,
        }
    }

    #[pure]
    pub fn allowance(&self, owner: AccountId, spender: AccountId) -> u128 {
        match self.allowances.get((owner, spender)) {
            Some(allowance) => allowance,
            None => 0,
        }
    }

    #[ensures(
        if (old(self.balance_of(caller)) >= value && to != caller) {
            forall(|acct_id: AccountId|
                if acct_id == to {
                    self.balance_of(acct_id) == old(self.balance_of(acct_id)) + value
                } else if acct_id == caller {
                    self.balance_of(acct_id) == old(self.balance_of(acct_id)) - value
                } else {
                    self.balance_of(acct_id) == old(self.balance_of(acct_id))
                }
            )
        } else {
            forall(|acct_id: AccountId|
                self.balance_of(acct_id) == old(self.balance_of(acct_id))
            )
        }
    )]
    #[ensures(forall(|a1: AccountId, a2: AccountId|
        self.allowance(a1, a2) == old(self.allowance(a1, a2))
    ))]
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
        if from_balance == value {
            self.balances.remove(caller);
        } else {
            self.balances.insert(caller, from_balance - value);
        }
        let to_balance = self.balance_of(to);
        self.balances.insert(to, to_balance + value);
        Ok(())
    }

    /// Transfers `value` tokens from `from` to `to`, but using the allowance
    /// granted by `from` to `caller`.
    #[ensures(if (
        old(self.balance_of(from)) >= value &&
        to != from &&
        (caller == from || old(self.allowance(from, caller)) >= value)
    ) {
        forall(|acct_id: AccountId|
            if acct_id == to {
                self.balance_of(acct_id) == old(self.balance_of(acct_id)) + value
            } else if acct_id == from {
                self.balance_of(acct_id) == old(self.balance_of(acct_id)) - value
            } else {
                self.balance_of(acct_id) == old(self.balance_of(acct_id))
            }
        )
    } else {
        forall(|acct_id: AccountId|
            self.balance_of(acct_id) == old(self.balance_of(acct_id))
        )
    }
)]
    #[ensures(
    if (
        old(self.balance_of(from)) >= value &&
        to != from &&
        caller != from &&
        old(self.allowance(from, caller)) >= value
    ) {
        forall(|a1: AccountId, a2: AccountId|
            if (a1 == from && a2 == caller) {
                self.allowance(a1, a2) == old(self.allowance(a1, a2)) - value
            } else {
                self.allowance(a1, a2) == old(self.allowance(a1, a2))
            }
        )
    } else {
        forall(|a1: AccountId, a2: AccountId|
            self.allowance(a1, a2) == old(self.allowance(a1, a2))
        )
    }
)]
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
        let from_balance = self.balance_of(from);
        if from_balance < value {
            return Err(PSP22Error::InsufficientBalance);
        }

        if allowance == value {
            self.allowances.remove((from, caller));
        } else {
            self.allowances.insert((from, caller), allowance - value);
        }

        if from_balance == value {
            self.balances.remove(from);
        } else {
            self.balances.insert(from, from_balance - value);
        }
        let to_balance = self.balance_of(to);
        // Total supply is limited by u128.MAX so no overflow is possible
        self.balances.insert(to, to_balance + value);
        Ok(())
    }

    /// Sets a new `value` for allowance granted by `owner` to `spender`.
    /// Overwrites the previously granted value.
    #[ensures(
        forall(|a1: AccountId, a2: AccountId|
            if (owner != spender && a1 == owner && a2 == spender) {
                self.allowance(a1, a2) == value
            } else {
                self.allowance(a1, a2) == old(self.allowance(a1, a2))
            }
        )
    )]
    #[ensures(
        forall(|a1: AccountId|
            self.balance_of(a1) == old(self.balance_of(a1))
        )
    )]
    pub fn approve(
        &mut self,
        owner: AccountId,
        spender: AccountId,
        value: u128,
    ) -> Result<(), PSP22Error> {
        if owner == spender {
            return Ok(());
        }
        if value == 0 {
            self.allowances.remove((owner, spender));
        } else {
            self.allowances.insert((owner, spender), value);
        }
        Ok(())
    }

    /// Increases the allowance granted by `owner` to `spender` by `delta_value`.
    #[ensures(
        forall(|a1: AccountId, a2: AccountId|
            self.allowance(a1, a2) == if (owner != spender && a1 == owner && a2 == spender) {
                old(self.allowance(a1, a2)) + delta_value
            } else {
                old(self.allowance(a1, a2))
            }
        )
    )]
    #[ensures(
        forall(|a1: AccountId|
            self.balance_of(a1) == old(self.balance_of(a1))
        )
    )]
    pub fn increase_allowance(
        &mut self,
        owner: AccountId,
        spender: AccountId,
        delta_value: u128,
    ) -> Result<(), PSP22Error> {
        if owner == spender || delta_value == 0 {
            return Ok(());
        }
        let allowance = self.allowance(owner, spender) + delta_value;
        self.allowances.insert((owner, spender), allowance);
        Ok(())
    }

    /// Decreases the allowance granted by `owner` to `spender` by `delta_value`.
    #[ensures(
        forall(|a1: AccountId, a2: AccountId|
            self.allowance(a1, a2) == if (
                old(self.allowance(a1, a2)) >= delta_value &&
                owner != spender &&
                a1 == owner &&
                a2 == spender
            ) {
                old(self.allowance(a1, a2)) - delta_value
            } else {
                old(self.allowance(a1, a2))
            }
        )
    )]
    #[ensures(
        forall(|a1: AccountId|
            self.balance_of(a1) == old(self.balance_of(a1))
        )
    )]
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
        let amount = allowance - delta_value;
        if amount == 0 {
            self.allowances.remove((owner, spender));
        } else {
            self.allowances.insert((owner, spender), amount);
        }
        Ok(())
    }

    /// Mints `value` of new tokens to `to` account.
    pub fn mint(&mut self, to: AccountId, value: u128) -> Result<(), PSP22Error> {
        if value == 0 {
            return Ok(());
        }
        if u128::MAX - self.total_supply < value {
            return Err(PSP22Error::Custom(String::from(
                "Max PSP22 supply exceeded. Max supply limited to 2^128-1.",
            )));
        }
        let new_supply = self.total_supply + value;
        self.total_supply = new_supply;
        prusti_assume!(u128::MAX - self.balance_of(to) >= value);
        let new_balance = self.balance_of(to) + value;
        self.balances.insert(to, new_balance);
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
        if balance == value {
            self.balances.remove(from);
        } else {
            self.balances.insert(from, balance - value);
        }
        prusti_assume!(self.total_supply >= value);
        self.total_supply = self.total_supply - value;
        Ok(())
    }
}
