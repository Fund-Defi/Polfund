#![cfg_attr(not(feature = "std"), no_std, no_main)]


#[ink::contract]
mod polfund {
    use ink::storage::Mapping;
    use ink::prelude::vec::Vec;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Polfund {
        /// Stores a single `bool` value on the storage.
        value: bool,
    }

    #[ink(storage)]
    pub struct CrowdFundingDeFi {
        name : String,
        location : String,
        owner: AccountId,
        campaigns: StorageHashMap<AccountId, Campaign>,
        contributors: StorageHashMap<(AccountId, AccountId), Balance>,
        loans: StorageVec<Loan>,
        locked_assets: StorageHashMap<AccountId, Balance>,
        staked_assets: StorageHashMap<AccountId, Balance>,
        messages: StorageHashMap<(AccountId, AccountId), Vec<Message>>,
    }
    

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    pub struct Loan {
        borrower: AccountId,
        amount: Balance,
        interest_rate: u32,
        duration: u32,
        collateral: Balance,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    pub struct Message {
        sender: AccountId,
        receiver : AccountId,
        content: String,
    }


    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    pub struct Campaign {
        creator: AccountId,
        target_amount: Balance,
        current_amount: Balance,
        deadline: u64,
        is_closed: bool,
    }


    impl Polfund {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        // #[ink(constructor)]
        // pub fn new(init_value: bool) -> Self {
        //     Self { value: init_value }
        // }

        #[ink(constructor)]
        pub fn new(_name : String, _location : String) -> Self {
            Self {
                name : _name,
                location : _location,
                owner: Self::env().caller(),
                campaigns: StorageHashMap::new(),
                backers: StorageHashMap::new(),
                loans: StorageVec::new(),
                locked_assets: StorageHashMap::new(),
                staked_assets: StorageHashMap::new(),
                messages: StorageHashMap::new(),
            }
        }

        #[ink(message)]
        pub fn create_campaign(&mut self, target_amount: Balance, deadline: u64) {
            let creator = self.env().caller();
            let campaign = Campaign {
                creator,
                target_amount,
                current_amount: 0,
                deadline,
                is_closed: false,
            };
            self.campaigns.insert(creator, campaign);
        }


        #[ink(message)]
        pub fn contribute_to_campaign(&mut self, campaign_creator: AccountId, amount: Balance) {
            let contributor = self.env().caller();
            let campaign = self.campaigns.get_mut(&campaign_creator).expect("Campaign not found");

            assert!(
                !campaign.is_closed,
                "You cannot contribute to a closed campaign!"
            );
            assert!(
                self.env().block_timestamp() < campaign.deadline,
                "Campaign deadline passed "
            );

            campaign.current_amount += amount;
            let key = (campaign_creator, contributor);
            let contributor_balance = self.contributors.entry(key).or_insert(0);
            *contributor_balance += amount;
        }

        #[ink(message)]
        pub fn close_campaign(&mut self, campaign_creator: AccountId) {
            let campaign = self.campaigns.get_mut(&campaign_creator).expect("Campaign not found");
            assert!(
                self.env().block_timestamp() >= campaign.deadline,
                "Cannot close campaign before deadline"
            ); // Depnding though
            campaign.is_closed = true;
        }


        // Message feat
        #[ink(message)]
        pub fn send_message(&mut self, recipient: AccountId, content: String) {
            let sender = self.env().caller();
            let key = (sender, recipient);
            let message = Message { sender, receiver, content };
            let message_list = self.messages.entry(key).or_insert(Vec::new());
            message_list.push(message);
        }

        #[ink(message)]
        pub fn get_messages(&self, sender: AccountId, recipient: AccountId) -> Vec<Message> {
            self.messages.get(&(sender, recipient)).cloned().unwrap_or_default()
        }


    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        // #[ink::test]
        // fn default_works() {
        //     let polfund = Polfund::default();
        //     assert_eq!(polfund.get(), false);
        // }

        // /// We test a simple use case of our contract.
        // #[ink::test]
        // fn it_works() {
        //     let mut polfund = Polfund::new(false);
        //     assert_eq!(polfund.get(), false);
        //     polfund.flip();
        //     assert_eq!(polfund.get(), true);
        // }
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = PolfundRef::default();

            // When
            let contract_account_id = client
                .instantiate("polfund", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<PolfundRef>(contract_account_id.clone())
                .call(|polfund| polfund.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = PolfundRef::new(false);
            let contract_account_id = client
                .instantiate("polfund", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<PolfundRef>(contract_account_id.clone())
                .call(|polfund| polfund.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<PolfundRef>(contract_account_id.clone())
                .call(|polfund| polfund.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get = build_message::<PolfundRef>(contract_account_id.clone())
                .call(|polfund| polfund.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
