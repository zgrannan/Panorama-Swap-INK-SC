mod data;
mod errors;
mod testing;
mod traits;

pub use data::PSP22Data;
pub use errors::PSP22Error;
pub use traits::{PSP22Burnable, PSP22Metadata, PSP22Mintable, PSP22};

pub type AccountId = u32;
pub struct Env(AccountId);

// An example code of a smart contract using PSP22Data struct to implement
// the functionality of PSP22 fungible token.
//
// Any contract can be easily enriched to act as PSP22 token by:
// (1) adding PSP22Data to contract storage
// (2) properly initializing it
// (3) defining the correct Transfer and Approval events
// (4) implementing PSP22 trait based on PSP22Data methods
// (5) properly emitting resulting events
//
// It is a good practice to also implement the optional PSP22Metadata extension (6)
// and include unit tests (7).
mod token {
    use crate::{AccountId, Env, PSP22Data, PSP22Error, PSP22Metadata, PSP22};
    use crate::data::{Allowance, Money};
    use prusti_contracts::*;

    impl Env {
        #[pure]
        pub fn caller(&self) -> AccountId {
            self.0
        }
        fn emit_event<T>(&self, event: T) {}
    }

    pub struct Token {
        data: PSP22Data, // (1)
        name: Option<String>,
        symbol: Option<String>,
        decimals: u8,
        env: Env,
    }

    impl Token {

        #[pure]
        pub fn env(&self) -> &Env {
            &self.env
        }

        pub fn new(
            supply: u128,
            name: Option<String>,
            symbol: Option<String>,
            decimals: u8,
            caller: AccountId,
        ) -> Self {
            Self {
                env: Env(caller),
                data: PSP22Data::new(supply, caller), // (2)
                name,
                symbol,
                decimals,
            }
        }
    }

    // (3)
    pub struct Approval {
        owner: AccountId,
        spender: AccountId,
        amount: u128,
    }

    // (3)
    pub struct Transfer {
        from: Option<AccountId>,
        to: Option<AccountId>,
        value: u128,
    }

    // (4)
    impl PSP22 for Token {

        #[pure]
        fn env(&self) -> &Env {
            &self.env
        }

        fn total_supply(&self) -> u128 {
            self.data.total_supply()
        }

        #[pure]
        fn balance_of(&self, owner: AccountId) -> u128 {
            self.data.balance_of(owner)
        }

        #[pure]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> u128 {
            self.data.allowance(owner, spender)
        }

        fn transfer(
            &mut self,
            to: AccountId,
            value: u128,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            produce!(resource(Money(self.env.caller()), value));
            self.data.transfer(self.env.caller(), to, value)
        }

        fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: u128,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            produce!(resource(Money(from), value));
            produce!(resource(Allowance(from, self.env.caller()), value));
            self.data.transfer_from(self.env.caller(), from, to, value)
        }

        fn approve(&mut self, spender: AccountId, value: u128) -> Result<(), PSP22Error> {
            produce!(resource(Allowance(self.env().caller(), spender), self.allowance(
                self.env().caller(),
                spender
            )));
            self.data.approve(self.env.caller(), spender, value)
        }

        fn increase_allowance(
            &mut self,
            spender: AccountId,
            delta_value: u128,
        ) -> Result<(), PSP22Error> {
            self.data
                .increase_allowance(self.env.caller(), spender, delta_value)
        }

        fn decrease_allowance(
            &mut self,
            spender: AccountId,
            delta_value: u128,
        ) -> Result<(), PSP22Error> {
            produce!(resource(Allowance(self.env().caller(), spender), delta_value));
            self.data.decrease_allowance(self.env.caller(), spender, delta_value)
        }
    }

    // (6)
    impl PSP22Metadata for Token {
        fn token_name(&self) -> Option<String> {
            self.name.clone()
        }
        fn token_symbol(&self) -> Option<String> {
            self.symbol.clone()
        }
        fn token_decimals(&self) -> u8 {
            self.decimals
        }
    }

    // (7)
    #[cfg(test)]
    mod tests {
        crate::tests!(Token, (|supply| Token::new(supply, None, None, 0)));
    }
}
