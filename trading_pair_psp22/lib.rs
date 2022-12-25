#![cfg_attr(not(feature = "std"), no_std)]

pub use self::trading_pair_psp22::{
	TradingPairPsp22,
	TradingPairPsp22Ref,
};


#[ink::contract]
pub mod trading_pair_psp22 {
    
    
    use openbrush::{
        contracts::{

            traits::psp22::PSP22Ref,
        },
    };

    use ink::storage::Mapping;
        
    
    #[ink(storage)]
    pub struct TradingPairPsp22 {

        //Number of overall transactions (Not including LP provision)
        transasction_number: i64,
        //PSP22 1 token contract address
        psp22_token1_address: AccountId,
        //PSP22 2 token contract address
        psp22_token2_address: AccountId,
        //LP fee
        fee: Balance,
        //Total LP token supply
        total_supply: Balance,
        //LP token balances of LP providers
        balances: Mapping<AccountId, Balance>,
        //PANX contract address
        panx_contract: AccountId,
        //Accounts LP tokens allowances
        lp_tokens_allowances: Mapping<(AccountId,AccountId), Balance>,
        //Valut account id to transfer trader's fee to
        vault: AccountId,
        //Trader's fee
        traders_fee:Balance
    }


    impl TradingPairPsp22 {
        /// Creates a new instance of trading pair psp22 contract.
        #[ink(constructor)]
        pub fn new(psp22_token1_contract:AccountId,psp22_token2_contract:AccountId, fee: Balance,panx_contract:AccountId,vault:AccountId) -> Self {
            

            let transasction_number:i64 = 0;
            let psp22_token1_address = psp22_token1_contract;
            let psp22_token2_address = psp22_token2_contract;
            let balances = Mapping::default();
            let lp_tokens_allowances = Mapping::default();
            let total_supply = 0;
            let traders_fee:Balance = 25000000000 / 10u128.pow(12);

            Self {
                transasction_number,
                psp22_token1_address,
                psp22_token2_address,
                fee,
                total_supply,
                balances,
                panx_contract,
                lp_tokens_allowances,
                vault,
                traders_fee
            }

            
        }

       ///function to provide liquidity to a PSP22/PSP22 trading pair contract.
       #[ink(message,payable)]
       pub fn provide_to_pool(&mut self,psp22_token1_deposit_amount:Balance,psp22_token2_deposit_amount:Balance,excpeted_lp_tokens:Balance,slippage:Balance)  {

           let mut shares:Balance = 0;

           let caller_current_balance_token1 = PSP22Ref::balance_of(&self.psp22_token1_address, self.env().caller());

           //making sure that caller current PSP22 1 token balance is greater than the deposit amount.
           if caller_current_balance_token1 < psp22_token1_deposit_amount {
            panic!(
                 "Caller does not have enough PSP22_1 tokens to provide to pool,
                 kindly lower the amount of deposited PSP22_1 tokens."
            )
            }

           let caller_current_balance_token2 = PSP22Ref::balance_of(&self.psp22_token2_address, self.env().caller());

           //making sure that caller current PSP22 2 token balance is greater than the deposit amount.
           if caller_current_balance_token2 < psp22_token2_deposit_amount {
            panic!(
                 "Caller does not have enough PSP22_2 tokens to provide to pool,
                 kindly lower the amount of deposited PSP22_2 tokens."
            )
            }

           let contract_token1_allowance = PSP22Ref::allowance(&self.psp22_token1_address, self.env().caller(),Self::env().account_id());

           //making sure that the trading pair contract has enough PSP22 1 token allowance.
           if contract_token1_allowance < psp22_token1_deposit_amount {
            panic!(
                 "Trading pair does not have enough allowance to transact,
                 make sure you approved the amount of deposited PSP22_1 tokens."
            )
            }

           let contract_token2_allowance = PSP22Ref::allowance(&self.psp22_token2_address, self.env().caller(),Self::env().account_id());

           //making sure that the trading pair contract has enough PSP22 2 token allowance.
           if contract_token2_allowance < psp22_token2_deposit_amount {
            panic!(
                 "Trading pair does not have enough allowance to transact,
                 make sure you approved the amount of deposited PSP22_2 tokens."
            )
            }

           let contract_psp22_1_starting_balance = PSP22Ref::balance_of(&self.psp22_token1_address, Self::env().account_id());

           //cross contract call to psp22 1 token contract to transfer psp22 1 token to the trading pair contract.
           if PSP22Ref::transfer_from_builder(&self.psp22_token1_address, self.env().caller(), Self::env().account_id(), psp22_token1_deposit_amount, ink::prelude::vec![]).call_flags(ink::env::CallFlags::default().set_allow_reentry(true)).fire().expect("Transfer failed").is_err(){
            panic!(
                "Error in PSP22_1 transferFrom cross contract call function, kindly re-adjust your deposited PSP22_1 tokens amount."
           )
           }

           let contract_psp22_1_closing_balance = PSP22Ref::balance_of(&self.psp22_token1_address, Self::env().account_id());

           let mut actual_psp22_1_deposit_amount:Balance = 0;
        
           //calculating the actual amount of PSP22 1 token  deposited amount (some PSP22 tokens might have internal tax)
           match contract_psp22_1_closing_balance.checked_sub(contract_psp22_1_starting_balance) {
                Some(result) => {
                    actual_psp22_1_deposit_amount = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

           let contract_psp22_2_starting_balance = PSP22Ref::balance_of(&self.psp22_token2_address, Self::env().account_id());

           //cross contract call to psp22 2 token contract to transfer psp22 2 token to the trading pair contract.
           if PSP22Ref::transfer_from_builder(&self.psp22_token2_address, self.env().caller(), Self::env().account_id(), psp22_token2_deposit_amount, ink::prelude::vec![]).call_flags(ink::env::CallFlags::default().set_allow_reentry(true)).fire().expect("Transfer failed").is_err(){
            panic!(
                "Error in PSP22_2 transferFrom cross contract call function, kindly re-adjust your deposited PSP22_2 tokens amount."
           )
           }

           let contract_psp22_2_closing_balance = PSP22Ref::balance_of(&self.psp22_token1_address, Self::env().account_id());
          
           let mut actual_psp22_2_deposit_amount:Balance = 0;

           //calculating the actual amount of PSP22 2 token deposited amount (some PSP22 tokens might have internal tax)
           match contract_psp22_2_closing_balance.checked_sub(contract_psp22_2_starting_balance) {
                Some(result) => {
                    actual_psp22_2_deposit_amount = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

           //if its the pool first deposit
           if self.total_supply == 0 {
            
            //calculating the amount of shares to give to the provider if its the first LP deposit overall
            shares = 1000u128 * 10u128.pow(12);

            }

           //if its not the first LP deposit
           if self.total_supply > 0{

            //calculating the amount of shares to give to the provider if its not the LP deposit
            match (actual_psp22_1_deposit_amount * self.total_supply).checked_div(self.get_psp22_token1_reserve() - actual_psp22_1_deposit_amount) {
                Some(result) => {
                    shares = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

             
           }

           //validating that shares is greater than 0
           if shares <= 0 {

            PSP22Ref::transfer(&self.psp22_token1_address, self.env().caller(), actual_psp22_1_deposit_amount, ink::prelude::vec![]).unwrap_or_else(|error| {
                panic!(
                    "Failed to transfer PSP22 1 tokens to caller : {:?}",
                    error
                )
            });

            PSP22Ref::transfer(&self.psp22_token2_address, self.env().caller(), actual_psp22_2_deposit_amount, ink::prelude::vec![]).unwrap_or_else(|error| {
                panic!(
                    "Failed to transfer PSP22 2 tokens to caller : {:?}",
                    error
                )
            });
            panic!(
                 "Expected given liquidity pool SHARES are equal to 0,
                 cannot proceed with liquidity pool provision."
            )
            }

           //function to return the percentage diff between the expected lp token that was shown in the front-end and the final shares amount.
           let percentage_diff = self.check_diffrenece(excpeted_lp_tokens,shares);

           //validating slippage
           if percentage_diff > slippage.try_into().unwrap() {

            PSP22Ref::transfer(&self.psp22_token1_address, self.env().caller(), actual_psp22_1_deposit_amount, ink::prelude::vec![]).unwrap_or_else(|error| {
                panic!(
                    "Failed to transfer PSP22 1 tokens to caller : {:?}",
                    error
                )
            });

            PSP22Ref::transfer(&self.psp22_token2_address, self.env().caller(), actual_psp22_2_deposit_amount, ink::prelude::vec![]).unwrap_or_else(|error| {
                panic!(
                    "Failed to transfer PSP22 2 tokens to caller : {:?}",
                    error
                )
            });


            panic!(
                "The percentage difference is bigger than the given slippage,
                kindly re-adjust the slippage settings."
            )
            }

           //caller current shares (if any)
           let current_shares = self.get_lp_token_of(self.env().caller());

            let new_caller_shares:Balance;

            //calculating the current caller shares with the new provided shares.
             match current_shares.checked_add(shares) {
                 Some(result) => {
                     new_caller_shares = result;
                 }
                 None => {
                     panic!("overflow!");
                 }
             };



           //increasing LP balance of caller (mint)
           self.balances.insert(self.env().caller(), &(new_caller_shares));
           //adding to over LP tokens (mint)
           self.total_supply += shares;



       }

       ///function to withdraw specific amount of LP share tokens.
       #[ink(message,payable)]
       pub fn withdraw_specific_amount(&mut self, shares: Balance)  {
          
           //caller address
           let caller = self.env().caller();

           //caller total LP shares
           let caller_shares = self.balances.get(&caller).unwrap_or(0);

           //Validating that the caller has the given number of shares.
           if caller_shares < shares {
            panic!(
                 "Caller does not have enough liquidity pool SHARES to withdraw,
                  kindly lower the liquidity pool SHARES withdraw amount."
            )
            }

           //Amount of psp22 token1 to give to the caller
           let psp22_token1_amount_to_give = self.get_psp22_token1_withdraw_tokens_amount(shares);
           //Amount of psp22 token1 to give to the caller
           let psp22_token2_amount_to_give = self.get_psp22_token2_withdraw_tokens_amount(shares);
           
           //cross contract call to PSP22 token1 contract to transfer PSP22 token1 to the caller
           PSP22Ref::transfer(&self.psp22_token1_address, caller, psp22_token1_amount_to_give, ink::prelude::vec![]).unwrap_or_else(|error| {
            panic!(
                "Failed to transfer PSP22 1 tokens to caller : {:?}",
                error
            )
            });
           
           //cross contract call to PSP22 token2 contract to transfer PSP22 token2 to the caller
           PSP22Ref::transfer(&self.psp22_token2_address, caller, psp22_token2_amount_to_give, ink::prelude::vec![]).unwrap_or_else(|error| {
            panic!(
                "Failed to transfer PSP22 2 tokens to caller : {:?}",
                error
            )
            });

           //reducing caller LP token balance_caller_shares
           self.balances.insert(caller, &(caller_shares - shares));
           //reducing over LP token supply (burn)
           self.total_supply -= shares;



       }

        
        ///funtion to get amount of withdrable PSP22/PSP22 tokens by given number of LP shares.
        #[ink(message)]
        pub fn get_withdraw_tokens_amount(&self, share_amount: Balance) -> (Balance,Balance) {

            
            let mut amount_of_psp22_token1_to_give:Balance;

            //calculating the amount of PSP22 tokens 1 to give to the caller
            match (share_amount * self.get_psp22_token1_reserve()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_token1_to_give = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            //calculating the amount of PSP22 tokens 2 to give to the caller
            let mut amount_of_psp22_token2_to_give:Balance;

            match (share_amount * self.get_psp22_token2_reserve()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_token2_to_give = result;
                }
                None => {
                    panic!("overflow!");
                }
            };
        

            (amount_of_psp22_token1_to_give,amount_of_psp22_token2_to_give)
        
        }


        ///function to get the amount of withdrawable PSP22 token1 by given shares.
        #[ink(message)]
        pub fn get_psp22_token1_withdraw_tokens_amount(&self, share_amount: Balance) -> Balance {

       
            let amount_of_psp22_token1_to_give:Balance;

            //calculating the amount of PSP22 tokens 1 to give to the caller
            match (share_amount * self.get_psp22_token1_reserve()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_token1_to_give = result;
                }
                None => {
                    panic!("overflow!");
                }
            };
        

            amount_of_psp22_token1_to_give
        
        }

        ///function to get the amount of withdrawable PSP22 token2 by given LP shares.
        #[ink(message)]
        pub fn get_psp22_token2_withdraw_tokens_amount(&self, share_amount: Balance) -> Balance {

        
            //calculating the amount of PSP22 tokens 2 to give to the caller
            let amount_of_psp22_token2_to_give:Balance;

            match (share_amount * self.get_psp22_token2_reserve()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_token2_to_give = result;
                }
                None => {
                    panic!("overflow!");
                }
            };
        

            amount_of_psp22_token2_to_give
        
        
        }

        
        ///function to get caller pooled PSP22 token1 and PSP22 token2 amounts
        #[ink(message)]
        pub fn get_account_locked_tokens(&self,account_id:AccountId) -> (Balance,Balance) {
           
            //account address
            let caller = account_id;
            //get account LP tokens 
            let caller_shares:Balance = self.balances.get(&caller).unwrap_or(0);


            let mut amount_of_psp22_token1_to_give:Balance = 0;

            let mut amount_of_psp22_token2_to_give:Balance = 0;

            if caller_shares <= 0 {

                return (amount_of_psp22_token1_to_give,amount_of_psp22_token2_to_give)
                 
            }

            //calculating the amount of locked PSP22 token 1 of given caller
            match (caller_shares * self.get_psp22_token1_reserve()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_token1_to_give = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            //calculating the amount of locked PSP22 token 2 of given caller
            match (caller_shares * self.get_psp22_token2_reserve()).checked_div(self.total_supply) {
                Some(result) => {
                    amount_of_psp22_token2_to_give = result;
                }
                None => {
                    panic!("overflow!");
                }
            };
        

            (amount_of_psp22_token1_to_give,amount_of_psp22_token2_to_give)

            
        }

        //function to get expected amount of LP shares.
        #[ink(message,payable)]
        pub fn get_expected_lp_token_amount(&self,psp22_token1_deposit_amount:Balance) -> Balance {

           //init LP shares variable (shares to give to caller)
           let mut shares:Balance = 0;
           
           //if its the caller first deposit 
           if self.total_supply == 0 {

            //calculating the amount of shares to give to the provider if its the first LP deposit overall
            shares = 1000u128 * 10u128.pow(12);

           }
           
           //if its not the first LP deposit
           if self.total_supply > 0{

            //calculating the amount of shares to give to the provider if its not the LP deposit
            match (psp22_token1_deposit_amount * self.total_supply).checked_div(self.get_psp22_token1_reserve()) {
                Some(result) => {
                    shares = result;
                }
                None => {
                    panic!("overflow!");
                }
            };
             
           }

            shares
            
        }
 

        ///function to get the amount of PSP22 token2 given for 1 PSP22 token1
	    #[ink(message)]
        pub fn get_price_for_one_psp22_token1(&self)-> Balance {
            
            //formula to calculate the price
            let amount_out = self.get_est_price_psp22_token1_to_psp22_token2(1u128 * 10u128.pow(12));

            return amount_out

        }

        ///function to get the amount of PSP22 token1 given for 1 PSP22 token2
	    #[ink(message)]
        pub fn get_price_for_one_psp22_token2(&self)-> Balance {
            


            //formula to calculate the price
            let amount_out:Balance = self.get_est_price_psp22_token2_to_psp22_token1(1u128 * 10u128.pow(12));

            return amount_out

        }

        ///function to get the amount of PSP22 token2 the caller will get for given PSP22 token1 amount
        #[ink(message)]
        pub fn get_est_price_psp22_token1_to_psp22_token2(&self, amount_in: Balance)-> Balance {


            let caller_current_balance = PSP22Ref::balance_of(&self.panx_contract, self.env().caller());

            let actual_fee:Balance;

            //calculating the actual LP fee 
            match self.fee.checked_div(10u128.pow(12)) {
                Some(result) => {
                    actual_fee = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            
            let mut amount_in_with_lp_fees:Balance;

            //reducting the LP fee from the deposited PSP22 1 tokens 
            match amount_in.checked_mul(100u128 - actual_fee) {
                Some(result) => {
                    amount_in_with_lp_fees = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

           //validating if caller has more than 3500 PANX to check if caller is eligible for the incentive program 
           if caller_current_balance >= 3500u128 * 10u128.pow(12){

            if self.fee  <= 1400000000000u128 {

                //reducting HALF of the LP fee from the PSP22 amount in, if the caller has more than 3500 PANX and the LP fee is less than 1.4%
                match amount_in.checked_mul(100u128 - (actual_fee / 2u128)) {
                    Some(result) => {
                        amount_in_with_lp_fees = result;
                    }
                    None => {
                        panic!("overflow!");
                    }
                };

            }

            if self.fee  > 1400000000000u128 {

                //reducting (LP fee - 1) of the LP fee from the PSP22 amount in, if the caller has more than 3500 PANX and the LP fee is more than 1.4%
                match amount_in.checked_mul(100u128 - (actual_fee - 1u128)) {
                    Some(result) => {
                        amount_in_with_lp_fees = result;
                    }
                    None => {
                        panic!("overflow!");
                    }
                };

            }
         }


            let amount_out:Balance;

            //calculating the final PSP22 2 token amount to transfer to the caller.
            match (amount_in_with_lp_fees * self.get_psp22_token2_reserve()).checked_div((self.get_psp22_token1_reserve() * 100) + amount_in_with_lp_fees) {
                Some(result) => {
                    amount_out = result;
                }
                None => {
                    panic!("overflow!");
                }
            };
            
            return amount_out                        

        }


        ///function to get the amount of PSP22 token1 the caller will get for given PSP22 token2 amount
        #[ink(message)]
        pub fn get_est_price_psp22_token2_to_psp22_token1(&self, amount_in: Balance)-> Balance {

            let caller_current_balance = PSP22Ref::balance_of(&self.panx_contract, self.env().caller());

            let actual_fee:Balance;

            //calculating the actual LP fee 
            match self.fee.checked_div(10u128.pow(12)) {
                Some(result) => {
                    actual_fee = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            let mut amount_in_with_lp_fees:Balance;

            //reducting the LP fee from the deposited PSP22 2 tokens 
            match amount_in.checked_mul(100u128 - actual_fee) {
                Some(result) => {
                    amount_in_with_lp_fees = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            //validating if caller has more than 3500 PANX
            if caller_current_balance >= 3500 * 10u128.pow(12){

                if self.fee  <= 1400000000000u128 {

                    match amount_in.checked_mul(100 - (actual_fee / 2u128)) {
                        Some(result) => {
                            amount_in_with_lp_fees = result;
                        }
                        None => {
                            panic!("overflow!");
                        }
                    };

                }
 
                if self.fee  > 1400000000000u128 {

                    match amount_in.checked_mul(100 - (actual_fee - 1u128)) {
                        Some(result) => {
                            amount_in_with_lp_fees = result;
                        }
                        None => {
                            panic!("overflow!");
                        }
                    };

                }
             }


            let amount_out:Balance;

            //calculating the final PSP22 1 token amount to transfer to the caller.
            match (amount_in_with_lp_fees * self.get_psp22_token1_reserve()).checked_div((self.get_psp22_token2_reserve() * 100) + amount_in_with_lp_fees){
                Some(result) => {
                    amount_out = result;
                }
                None => {
                    panic!("overflow!");
                }
            };
            
            return amount_out                        

        }

        ///function to get the estimated price impact for given psp22 token1 amount
        #[ink(message)]
        pub fn get_price_impact_psp22_token1_to_psp22_token2(&self,amount_in: Balance) -> Balance {
            
            let actual_fee:Balance;

            //calculating the actual LP fee
            match self.fee.checked_div(10u128.pow(12)) {
                Some(result) => {
                    actual_fee = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            //fetching the amount of PSP22 2 tokens the caller WOULD get if he would swap
            let psp22_token2_amount_out:Balance = self.get_est_price_psp22_token1_to_psp22_token2(amount_in);

            //reducting the LP fee from the PSP22 amount in
            let amount_in_with_lp_fees:Balance;

            match amount_in.checked_mul(100 - (self.fee / 10u128.pow(12))) {
                Some(result) => {
                    amount_in_with_lp_fees = result;
                }
                None => {
                    panic!("overflow!");
                }
            };


            let amount_out:Balance;

            //calculating the final PSP22 2 token amount to transfer to the caller.
            match (amount_in_with_lp_fees * (self.get_psp22_token2_reserve() - psp22_token2_amount_out)).checked_div(((self.get_psp22_token1_reserve() + amount_in_with_lp_fees ) * 100) + amount_in_with_lp_fees) {
                Some(result) => {
                    amount_out = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            
            return amount_out    

        }
        ///function to get the estimated price impact for given psp22 token2 amount
        #[ink(message,payable)]
        pub fn get_price_impact_psp22_token2_to_psp22_token1(&self,amount_in:Balance) -> Balance {

            let actual_fee:Balance;

            //calculating the actual LP fee
            match self.fee.checked_div(10u128.pow(12)) {
                Some(result) => {
                    actual_fee = result;
                }
                None => {
                    panic!("overflow!");
                }
            };
            
            //fetching the amount of PSP22 1 tokens the caller WOULD get if he would swap
            let psp22_token1_amount_out = self.get_est_price_psp22_token2_to_psp22_token1(amount_in);

            //calc the amount_in with current fees to transfer to the LP providers.
            let amount_in_with_lp_fees:Balance;

            match amount_in.checked_mul(100 - actual_fee) {
                Some(result) => {
                    amount_in_with_lp_fees = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            let amount_out:Balance;

            //calculating the final PSP22 1 token amount to transfer to the caller.
            match (amount_in_with_lp_fees * (self.get_psp22_token1_reserve() - psp22_token1_amount_out)).checked_div(((self.get_psp22_token2_reserve() + amount_in_with_lp_fees ) * 100) + amount_in_with_lp_fees) {
                Some(result) => {
                    amount_out = result;
                }
                None => {
                    panic!("overflow!");
                }
            };
            
            return amount_out  


        }

        
        ///function to swap psp22 token1 to psp22 token2
        #[ink(message)]
        pub fn swap_psp22_token1(&mut self,psp22_token1_amount_to_swap: Balance, amount_to_validate: Balance,slippage: Balance) {

            let caller_current_balance = PSP22Ref::balance_of(&self.psp22_token1_address, self.env().caller());

            //making sure caller has more or equal to the amount he transfers.
            if caller_current_balance < psp22_token1_amount_to_swap {
                panic!(
                    "Caller balance is lower than the amount of PSP22_1 token he wishes to trasnfer,
                    kindly lower the deposited PSP22_1 tokens amount."
                )
            }

            let contract_allowance = PSP22Ref::allowance(&self.psp22_token1_address, self.env().caller(),Self::env().account_id());

            //making sure trading pair contract has enough allowance.
            if contract_allowance < psp22_token1_amount_to_swap {
                panic!(
                    "Trading pair does not have enough allowance to transact,
                    make sure you approved the amount of deposited PSP22_1 tokens before swapping."
                )
            }
            
            //amount of PSP22 tokens 2 to give to caller before traders fee.
            let psp22_token2_amount_out_for_caller_before_traders_fee = self.get_est_price_psp22_token1_to_psp22_token2(psp22_token1_amount_to_swap);

            //Calculating the final amount of psp22 tokens 2 to give to the caller after reducing traders fee
            let actual_psp22_token2_amount_out_for_caller:Balance;

            match psp22_token2_amount_out_for_caller_before_traders_fee.checked_sub(psp22_token2_amount_out_for_caller_before_traders_fee * self.traders_fee) {
                Some(result) => {
                    actual_psp22_token2_amount_out_for_caller = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            //Calculating the amount to allocate to the vault account
            let psp22_token2_amount_out_for_vault:Balance;

            match psp22_token2_amount_out_for_caller_before_traders_fee.checked_sub(actual_psp22_token2_amount_out_for_caller) {
                Some(result) => {
                    psp22_token2_amount_out_for_vault = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            //percentage dif between given PSP22 2 tokens amount from front-end and acutal final PSP22 2 tokens amount
            let percentage_diff = self.check_diffrenece(amount_to_validate,psp22_token2_amount_out_for_caller_before_traders_fee);

            //Validating slippage
            if percentage_diff > slippage.try_into().unwrap() {
                panic!(
                    "The percentage difference is bigger than the given slippage,
                    kindly re-adjust the slippage settings."
                )
            }

            //cross contract call to PSP22 contract to transfer PSP22 to the Trading Pair contract (self)
            if PSP22Ref::transfer_from_builder(&self.psp22_token1_address, self.env().caller(), Self::env().account_id(), psp22_token1_amount_to_swap, ink::prelude::vec![]).call_flags(ink::env::CallFlags::default().set_allow_reentry(true)).fire().expect("Transfer failed").is_err(){
                panic!(
                    "Error in PSP22_1 transferFrom cross contract call function, kindly re-adjust your deposited PSP22_1 tokens."
               )
            }

            //fun to transfer PSP22 2 token to caller
            if PSP22Ref::transfer(&self.psp22_token2_address, self.env().caller(), actual_psp22_token2_amount_out_for_caller, ink::prelude::vec![]).is_err() {
                panic!(
                    "Error in PSP22_2 transfer cross contract call function, kindly re-adjust PSP22_1 deposit amount."
                )
            }

            //fun to transfer PSP22 2 token2 to vault
            if PSP22Ref::transfer(&self.psp22_token2_address, self.vault , psp22_token2_amount_out_for_vault, ink::prelude::vec![]).is_err() {
                panic!(
                    "Error in PSP22_2 transfer cross contract call function, kindly re-adjust PSP22_1 deposit amount."
                )
            }


            //increase num of trans
            self.transasction_number = self.transasction_number + 1;

        }


        ///function to swap psp22 token2 to psp22 token1
        #[ink(message,payable)]
        pub fn swap_psp22_token2(&mut self,psp22_token2_amount_to_swap: Balance, amount_to_validate: Balance,slippage: Balance) {
            
            let caller_current_balance = PSP22Ref::balance_of(&self.psp22_token2_address, self.env().caller());

            //making sure caller has more or equal to the amount he transfers.
            if caller_current_balance < psp22_token2_amount_to_swap {
                panic!(
                    "Caller balance is lower than the amount of PSP22_2 token he wishes to trasnfer,
                    kindly lower your deposited PSP22_2 tokens amount."
                )
            }

            let contract_allowance = PSP22Ref::allowance(&self.psp22_token2_address, self.env().caller(),Self::env().account_id());
            //making sure trading pair contract has enough allowance.
            if contract_allowance < psp22_token2_amount_to_swap {
                panic!(
                    "Trading pair does not have enough allowance to transact,
                    make sure you approved the amount of deposited PSP22_2 tokens before swapping."
                )
            }


            //amount of PSP22 tokens 1 to give to caller before traders fee.
            let psp22_token1_amount_out_for_caller_before_traders_fee = self.get_est_price_psp22_token2_to_psp22_token1(psp22_token2_amount_to_swap);

            let actual_psp22_token1_amount_out_for_caller:Balance;

            //calculating the final amount of psp22 tokens 1 to give to the caller after reducing traders fee.
            match psp22_token1_amount_out_for_caller_before_traders_fee.checked_sub(psp22_token1_amount_out_for_caller_before_traders_fee * self.traders_fee) {
                Some(result) => {
                    actual_psp22_token1_amount_out_for_caller = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            
            let psp22_token1_amount_out_for_vault:Balance;

            //calculating the amount to allocate to the vault account
            match psp22_token1_amount_out_for_caller_before_traders_fee.checked_sub(actual_psp22_token1_amount_out_for_caller) {
                Some(result) => {
                    psp22_token1_amount_out_for_vault = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            //percentage dif between given PSP22 1 token amount (from front-end) and acutal final PSP22 1 token amount
            let percentage_diff = self.check_diffrenece(amount_to_validate,psp22_token1_amount_out_for_caller_before_traders_fee);

            //Validating slippage
            if percentage_diff > slippage.try_into().unwrap() {
                panic!(
                    "The percentage difference is bigger than the given slippage,
                    kindly re-adjust the slippage settings."
                )
            }


            //cross contract call to PSP22 contract to transfer PSP22 to the Trading Pair contract (self)
            if PSP22Ref::transfer_from_builder(&self.psp22_token2_address, self.env().caller(), Self::env().account_id(), psp22_token2_amount_to_swap, ink::prelude::vec![]).call_flags(ink::env::CallFlags::default().set_allow_reentry(true)).fire().expect("Transfer failed").is_err(){
                panic!(
                    "Error in PSP22_2 transferFrom cross contract call function, kindly re-adjust your deposited PSP22_2 tokens."
               )
            }


            //function to transfer PSP22 1 token to caller
            if PSP22Ref::transfer(&self.psp22_token1_address, self.env().caller(), actual_psp22_token1_amount_out_for_caller, ink::prelude::vec![]).is_err() {
                panic!(
                    "Error in PSP22_1 transfer cross contract call function, kindly re-adjust PSP22_2 deposit amount."
                )
            }

            //function to transfer PSP22 1 token to vault
            if PSP22Ref::transfer(&self.psp22_token1_address, self.vault, psp22_token1_amount_out_for_vault, ink::prelude::vec![]).is_err() {
                panic!(
                    "Error in PSP22_1 transfer cross contract call function, kindly re-adjust PSP22_2 deposit amount."
                )
            }

            //increase num of trans
            self.transasction_number = self.transasction_number + 1;
            
        }
        
        ///function used to transfer LP share tokens from caller to recipient.
        #[ink(message)]
        pub fn transfer_lp_tokens(&mut self, recipient:AccountId,shares_to_transfer: Balance) {

            let caller = self.env().caller();

            let caller_shares:Balance = self.balances.get(&caller).unwrap_or(0);

            let recipient_shares:Balance = self.balances.get(&recipient).unwrap_or(0);
        
            if caller_shares < shares_to_transfer {
                panic!(
                    "Cannot transfer LP shares to recipient, caller balance is lower than the requested transfer amount."
                )
            }

            let new_caller_lp_balance:Balance;

            //calculating caller total LP share tokens amount after transfer
            match caller_shares.checked_sub(shares_to_transfer) {
                Some(result) => {
                    new_caller_lp_balance = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            let new_recipient_lp_balance:Balance;

            //calculating caller total LP share tokens amount after transfer
            match recipient_shares.checked_add(shares_to_transfer) {
                Some(result) => {
                    new_recipient_lp_balance = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            self.balances.insert(caller, &(new_caller_lp_balance));
 
            self.balances.insert(recipient, &(new_recipient_lp_balance));


        }

        ///function used to approve the amount of LP token shares for the spender to spend from owner.
        #[ink(message)]
        pub fn approve_lp_tokens(&mut self, spender:AccountId,shares_to_approve: Balance)  {

           let caller = self.env().caller();

           let caller_shares:Balance = self.balances.get(&caller).unwrap_or(0);

           if caller_shares < shares_to_approve {
            panic!(
                "Cannot approve LP tokens, owner LP token balance is lower than the given shares to approve."
            )
           }

           //overflow validation
           if shares_to_approve >= u128::MAX {
            panic!(
                "overflow!"
            )
           }

           self.lp_tokens_allowances.insert((caller,spender), &(shares_to_approve));

        }


        //function to transfer LP share tokens FROM owner TO receipent
        #[ink(message)]
        pub fn transfer_lp_tokens_from_to(&mut self,owner:AccountId,to:AccountId,shares_to_transfer: Balance)  {

           let spender = self.env().caller();

           let owner_shares:Balance = self.balances.get(&owner).unwrap_or(0);

           let to_shares:Balance = self.balances.get(&to).unwrap_or(0);

           let allowance:Balance = self.get_lp_tokens_allowance(owner,spender);

           if allowance < shares_to_transfer {
            panic!(
                "Cannot transfer LP shares to spender, allowance is lower than the requested transfer amount."
            )
           }

           if owner_shares < shares_to_transfer {
            panic!(
                "Cannot transfer LP shares to spender, caller balance is lower than the requested transfer amount."
            )
           }

           let new_owner_lp_balance:Balance;

           //calculating caller total LP share tokens amount after transfer
           match owner_shares.checked_sub(shares_to_transfer) {
               Some(result) => {
                   new_owner_lp_balance = result;
               }
               None => {
                   panic!("overflow!");
               }
           };

           let new_to_lp_balance:Balance;

           //calculating caller total LP share tokens amount after transfer
           match to_shares.checked_add(shares_to_transfer) {
               Some(result) => {
                new_to_lp_balance = result;
               }
               None => {
                   panic!("overflow!");
               }
           };

           let new_allowance:Balance;

           //calculating spender new allowance amount
           match allowance.checked_sub(shares_to_transfer) {
                Some(result) => {
                    new_allowance = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

           self.balances.insert(owner, &(new_owner_lp_balance));

           self.lp_tokens_allowances.insert((owner,spender), &(new_allowance));

           self.balances.insert(to, &(new_to_lp_balance));
    
         
        }


        //function to get the allowance of spender from the owner
        pub fn get_lp_tokens_allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.lp_tokens_allowances.get(&(owner,spender)).unwrap_or(0)
        }

        ///function to get AzeroTradingPair contract address (self)
        #[ink(message)]
        pub fn get_account_id(&self) -> AccountId {
            Self::env().account_id()
        }

        ///function to get AzeroTradingPair contract address (self)
        #[ink(message)]
        pub fn get_fee(&self) -> Balance {
            self.fee
        }
        
        ///funtion to fetch current price for one PSP22
        #[ink(message)]
        pub fn get_current_price(&self) -> Balance {
        
            self.get_est_price_psp22_token1_to_psp22_token2(100u128 * 10u128.pow(12))    
        }

        ///function to get total supply of LP shares
        #[ink(message)]
        pub fn get_total_supply(&self) -> Balance {
            self.total_supply
        }


        ///function to get shares of specific account
        #[ink(message)]
        pub fn get_lp_token_of(&self,account: AccountId) -> Balance {
            let account_balance = self.balances.get(&account).unwrap_or(0);
            account_balance
        }
        ///function to get contract PSP22 token2 reserve (self)
        #[ink(message)]
        pub fn get_psp22_token2_reserve(&self) -> Balance {
            let balance = PSP22Ref::balance_of(&self.psp22_token2_address, Self::env().account_id());
            balance
        }
        ///function to get contract PSP22 token1 reserve (self)
        #[ink(message)]
        pub fn get_psp22_token1_reserve(&self) -> Balance {
            
            let balance = PSP22Ref::balance_of(&self.psp22_token1_address, Self::env().account_id());
            balance
        }



    	#[ink(message)]
        pub fn get_transactions_num(&self) -> i64 {
            self.transasction_number

        }
        
        #[ink(message,payable)]
        pub fn check_diffrenece(&mut self,value1: Balance,value2: Balance) -> Balance {

            let abs_dif = value1.abs_diff(value2);

            let abs_dif_nominated = abs_dif * 10u128.pow(12);

            let diff:Balance;

            match 100u128.checked_mul(abs_dif_nominated / ((value1+value2) / 2)) {
                Some(result) => {
                    diff = result;
                }
                None => {
                    panic!("overflow!");
                }
            };

            diff
            
        }
 
    }
}