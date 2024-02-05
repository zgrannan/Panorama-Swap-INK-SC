mod data;
mod errors;
mod testing;
mod traits;

pub use data::{PSP22Data, PSP22Event};
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
    use prusti_contracts::*;
    use crate::{AccountId, PSP22Data, PSP22Error, PSP22Event, PSP22Metadata, PSP22, Env};

    impl Env {
        #[pure]
        fn caller(&self) -> AccountId {
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
        pub fn env(&self) -> &Env {
            &self.env
        }

        pub fn new(
            supply: u128,
            name: Option<String>,
            symbol: Option<String>,
            decimals: u8,
            caller: AccountId
        ) -> Self {
            Self {
                env: Env(caller),
                data: PSP22Data::new(supply, caller), // (2)
                name,
                symbol,
                decimals,
            }
        }

        // A helper function translating a vector of PSP22Events into the proper
        // ink event types (defined internally in this contract) and emitting them.
        // (5)
        fn emit_events(&self, events: Vec<PSP22Event>) {
            for event in events {
                match event {
                    PSP22Event::Transfer { from, to, value } => {
                        self.env().emit_event(Transfer { from, to, value })
                    }
                    PSP22Event::Approval {
                        owner,
                        spender,
                        amount,
                    } => self.env().emit_event(Approval {
                        owner,
                        spender,
                        amount,
                    }),
                }
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
        fn total_supply(&self) -> u128 {
            self.data.total_supply()
        }

        fn balance_of(&self, owner: AccountId) -> u128 {
            self.data.balance_of(owner)
        }

        fn allowance(&self, owner: AccountId, spender: AccountId) -> u128 {
            self.data.allowance(owner, spender)
        }

        fn transfer(
            &mut self,
            to: AccountId,
            value: u128,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let events = self.data.transfer(self.env().caller(), to, value)?;
            // self.emit_events(events);
            Ok(())
        }

        fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: u128,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let events = self
                .data
                .transfer_from(self.env().caller(), from, to, value)?;
            // self.emit_events(events);
            Ok(())
        }

        fn approve(&mut self, spender: AccountId, value: u128) -> Result<(), PSP22Error> {
            let events = self.data.approve(self.env().caller(), spender, value)?;
            self.emit_events(events);
            Ok(())
        }

        fn increase_allowance(
            &mut self,
            spender: AccountId,
            delta_value: u128,
        ) -> Result<(), PSP22Error> {
            let events = self
                .data
                .increase_allowance(self.env().caller(), spender, delta_value)?;
            self.emit_events(events);
            Ok(())
        }

        fn decrease_allowance(
            &mut self,
            spender: AccountId,
            delta_value: u128,
        ) -> Result<(), PSP22Error> {
            let events = self
                .data
                .decrease_allowance(self.env().caller(), spender, delta_value)?;
            self.emit_events(events);
            Ok(())
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
