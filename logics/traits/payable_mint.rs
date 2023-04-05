use ink_prelude::string::String as PreludeString;

use openbrush::{
    contracts::psp34::PSP34Error,
    traits::{
        AccountId,
        Balance,
    },
    modifiers
};

#[openbrush::wrapper]
pub type PayableMintRef = dyn PayableMint;

#[openbrush::trait_definition]
pub trait PayableMint {
    /// Mint one or more tokens
    #[ink(message, payable)]
    fn mint(&mut self, to: AccountId, mint_amount: u64) -> Result<(), PSP34Error>;

    /// Mint next available token for the caller
    #[ink(message, payable)]
    fn mint_next(&mut self) -> Result<(), PSP34Error>;

    /// Set new value for the baseUri
    #[ink(message)]
    fn set_base_uri(&mut self, uri: PreludeString) -> Result<(), PSP34Error>;

    /// Withdraws funds to contract owner
    #[ink(message)]
    fn withdraw(&mut self) -> Result<(), PSP34Error>;

    /// Set max number of tokens which could be minted per call
    #[ink(message)]
    fn set_max_mint_amount(&mut self, max_amount: u64) -> Result<(), PSP34Error>;

    /// Get URI from token ID
    #[ink(message)]
    fn token_uri(&self, token_id: u64) -> Result<PreludeString, PSP34Error>;

    /// Get max supply of tokens
    #[ink(message)]
    fn max_supply(&self) -> u128;

    /// Get token price
    #[ink(message)]
    fn price(&self) -> Balance;

    /// Get max number of tokens which could be minted per call
    #[ink(message)]
    fn get_max_mint_amount(&mut self) -> u64;

    #[ink(message)]
    #[modifiers(only_owner)]
    fn set_mint_end(&mut self, status: bool) -> Result<(), PSP34Error>;

    #[ink(message)]
    fn get_mint_end(&self) -> bool;

    #[ink(message)]
    fn get_is_account_minted(&self, account_id: AccountId) -> bool;
}