use ink_prelude::string::String as PreludeString;
use ink_prelude::vec::Vec;

use openbrush::{
    contracts::psp34::PSP34Error,
    traits::{AccountId, Balance},
};

#[openbrush::wrapper]
pub type LaunchpadRef = dyn Launchpad;

#[openbrush::trait_definition]
pub trait Launchpad {
    /// Mint one or more tokens
    #[ink(message, payable)]
    fn mint(&mut self, to: AccountId, mint_amount: u64) -> Result<Vec<u64>, PSP34Error>;

    /// Mint next available token for the caller
    #[ink(message, payable)]
    fn mint_next(&mut self) -> Result<u64, PSP34Error>;

    #[ink(message)]
    fn refund(&mut self, token_id: u64) -> Result<(), PSP34Error>;

    // Get refund amount for given token_id
    #[ink(message)]
    fn get_refund_amount(&self, token_id: u64) -> Balance;

    /// Set new value for the baseUri
    #[ink(message)]
    fn set_base_uri(&mut self, uri: PreludeString) -> Result<(), PSP34Error>;

    /// Withdraw funds to contract owner
    #[ink(message)]
    fn withdraw_launchpad(&mut self) -> Result<(), PSP34Error>;

    /// Withdraw funds to launchpad project
    #[ink(message)]
    fn withdraw_project(&mut self) -> Result<(), PSP34Error>;

    /// Set max number of tokens which could be minted per call
    #[ink(message)]
    fn set_max_mint_amount(&mut self, max_amount: u64) -> Result<(), PSP34Error>;

    /// Get URI from token ID
    #[ink(message)]
    fn token_uri(&self, token_id: u64) -> Result<PreludeString, PSP34Error>;

    /// Get max supply of tokens
    #[ink(message)]
    fn max_supply(&self) -> u64;

    /// Get token price
    #[ink(message)]
    fn price(&self) -> Balance;

    /// Get max number of tokens which could be minted per call
    #[ink(message)]
    fn get_max_mint_amount(&mut self) -> u64;
}
