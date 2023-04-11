#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod erc20 {
    use ink::storage::Mapping;

    /// Specify ERC-20 error type.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        InsufficientAllowance,
    }

    /// Specify the ERC-20 result type.
    pub type Result<T> = core::result::Result<T, Error, Vec>;

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }


    

    /// Create storage for a simple ERC-20 contract.
    #[ink(storage)]
    pub struct Erc20 {
        /// Total token supply.
        total_supply: Balance,
        /// Mapping from owner to number of owned tokens.
        balances: Mapping<AccountId, Balance>,
        allowances: Mapping<(AccountId, AccountId), Balance>,
        allowances_to_others: Mapping<AccountId, Balance>,
        allowances_from_others: Mapping<AccountId, Balance>,
    }

    impl Erc20 {
        /// Create a new ERC-20 contract with an initial supply.
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {

            
            let allowances = Mapping::default();
            let allowances_to_others = Mapping::default();
            let allowances_from_others = Mapping::default();
            let mut balances = Mapping::default();
            let caller = Self::env().caller();
            balances.insert(caller, &total_supply);

            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: total_supply,
            });

            Self {
                total_supply,
                balances,
                allowances,
                allowances_to_others,
                allowances_from_others
            }
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
        let owner = self.env().caller();
        self.allowances.insert((owner, spender), &value);
        // let tvalue = self.allowance_to(spender);
        // self.allowances_to_others.insert(spender, &(value + tvalue));
        // let fvalue = self.allowance_from(owner);
        // self.allowances_from_others.insert(owner, &(value + fvalue));

        self.env().emit_event(Approval {
            owner,
            spender,
            value,
        });

        Ok(())
        }

        #[ink(message)]
        pub fn decrease_approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
        let owner = self.env().caller();
        let current_allowance = self.allowance(owner, spender);
        if value > current_allowance {
            return Err(Error::InsufficientAllowance);
        }
        let new_allowance = current_allowance - value;
        self.allowances.insert((owner, spender), &new_allowance);

        self.env().emit_event(Approval {
            owner,
            spender,
            value: new_allowance,
        });

        Ok(())
        }


        #[ink(message)]
        pub fn increase_approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
        let owner = self.env().caller();
        let current_allowance = self.allowance(owner, spender);
        let new_allowance = current_allowance + value;
        
        self.allowances.insert((owner, spender), &new_allowance);

        self.env().emit_event(Approval {
            owner,
            spender,
            value: new_allowance,
        });

        Ok(())
        }



        /// Transfers tokens on the behalf of the `from` account to the `to account
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance(from, caller);
            if allowance < value {
                return Err(Error::InsufficientAllowance);
            }

            self.transfer_from_to(&from, &to, value)?;

            self.allowances.insert((from, caller), &(allowance - value));
            let tvalue = self.allowance_to(caller);
            self.allowances_to_others.insert(caller, &(tvalue - value));
            let fvalue = self.allowance_from(from);
            self.allowances_from_others.insert(from, &(fvalue - value));

            Ok(())
        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get((owner, spender)).unwrap_or_default()
        }

        #[ink(message)]
        pub fn allowance_from(&self, owner: AccountId) -> Balance {
            self.allowances_from_others.get(owner).unwrap_or_default()
        }

        #[ink(message)]
        pub fn allowance_to(&self, spender: AccountId) -> Balance {
            self.allowances_to_others.get(spender).unwrap_or_default()
        }




        /// Returns the total token supply.
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        /// Returns the account balance for the specified `owner`.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(owner).unwrap_or_default()
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let from = self.env().caller();
            self.transfer_from_to(&from, &to, value)
        }

        fn transfer_from_to(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            value: Balance,
         ) -> Result<()> {

            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                value,
            });

             let from_balance = self.balance_of(*from);
             if from_balance < value {
                 return Err(Error::InsufficientBalance)
             }
         
             self.balances.insert(&from, &(from_balance - value));
             let to_balance = self.balance_of(*to);
             self.balances.insert(&to, &(to_balance + value));
         
             Ok(())
         }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // We define some helper Accounts to make our tests more readable
        fn default_accounts() -> ink::env::test::DefaultAccounts<Environment> {
            ink::env::test::default_accounts::<Environment>()
        }

        fn alice() -> AccountId {
            default_accounts().alice
        }

        fn bob() -> AccountId {
            default_accounts().bob
        }

        #[ink::test]
        fn new_works() {
            let contract = Erc20::new(777);
            assert_eq!(contract.total_supply(), 777);
        }

        #[ink::test]
        fn balance_works() {
            let contract = Erc20::new(100);
            assert_eq!(contract.total_supply(), 100);
            assert_eq!(contract.balance_of(alice()), 100);
            assert_eq!(contract.balance_of(bob()), 0);
        }

        #[ink::test]
        fn transfer_works() {
            let mut contract = Erc20::new(100);
            assert_eq!(contract.balance_of(alice()), 100);
            assert!(contract.transfer(bob(), 10).is_ok());
            assert_eq!(contract.balance_of(bob()), 10);
            assert!(contract.transfer(bob(), 100).is_err());
        }

        #[ink::test]
        fn transfer_from_works() {
            let mut contract = Erc20::new(100);
            assert_eq!(contract.balance_of(alice()), 100);
            let _ = contract.approve(alice(), 20);
            let _ = contract.transfer_from(alice(), bob(), 10);
            assert_eq!(contract.balance_of(bob()), 10);
        }

        #[ink::test]
        fn allowances_works() {
            let mut contract = Erc20::new(100);
            assert_eq!(contract.balance_of(alice()), 100);
            let _ = contract.approve(alice(), 200);
            assert_eq!(contract.allowance(alice(), alice()), 200);

            assert!(contract.transfer_from(alice(), bob(), 50).is_ok());
            assert_eq!(contract.balance_of(bob()), 50);
            assert_eq!(contract.allowance(alice(), alice()), 150);

            assert!(contract.transfer_from(alice(), bob(), 100).is_err());
            assert_eq!(contract.balance_of(bob()), 50);
            assert_eq!(contract.allowance(alice(), alice()), 150);
        }

        #[ink::test]
        fn error_test() {
            let mut contract = Erc20::new(100);
            assert_eq!(contract.balance_of(alice()), 100);
            let _ = contract.transfer(bob(), 0);
            assert_eq!(contract.balance_of(bob()), 0);
            //assert!(contract.transfer(bob(), 100).is_err());
        }

        #[ink::test]
        fn decrease_allowance_works() {
            let mut contract = Erc20::new(100);
            assert_eq!(contract.balance_of(alice()), 100);
            let _ = contract.approve(alice(), 30);
            assert_eq!(contract.allowance(alice(), alice()), 30);
            let _ = contract.decrease_approve(alice(), 10);
            assert_eq!(contract.allowance(alice(), alice()), 20);
            assert!(contract.decrease_approve(alice(), 30).is_err());
            assert_eq!(contract.allowance(alice(), alice()), 20);
            let _ = contract.decrease_approve(alice(), 20);
            assert_eq!(contract.allowance(alice(), alice()), 0);

        }

        #[ink::test]
        fn increase_allowance_works() {
            let mut contract = Erc20::new(100);
            assert_eq!(contract.balance_of(alice()), 100);
            let _ = contract.approve(alice(), 30);
            assert_eq!(contract.allowance(alice(), alice()), 30);
            let _ = contract.increase_approve(alice(), 10);
            assert_eq!(contract.allowance(alice(), alice()), 40);

        }

        #[ink::test]
        fn accesss_allowances_works() {
            let mut contract = Erc20::new(100);
            assert_eq!(contract.balance_of(alice()), 100);

            let _ = contract.approve(alice(), 30);
            assert_eq!(contract.allowance(alice(), alice()), 30);
            assert_eq!(contract.allowance_to(alice()), 30);
            assert_eq!(contract.allowance_from(alice()), 30);

            let _ = contract.approve(bob(), 20);
            assert_eq!(contract.allowance_to(alice()), 30);
            assert_eq!(contract.allowance_from(alice()), 50);
            assert_eq!(contract.allowance_to(bob()), 20);

            assert!(contract.transfer_from(alice(), bob(), 10).is_ok());
            assert_eq!(contract.allowance_to(alice()), 20);
            assert_eq!(contract.allowance_from(alice()), 40);
            assert_eq!(contract.allowance_to(bob()), 20);

        }
    }
}


