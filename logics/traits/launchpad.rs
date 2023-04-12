use openbrush::{
    contracts::psp34::PSP34Error,
    traits::{AccountId, Balance, String},
};

#[openbrush::wrapper]
pub type LaunchpadRef = dyn Launchpad;

#[openbrush::trait_definition]
pub trait Launchpad {
    /// Mint one or more tokens
    #[ink(message, payable)]
    fn mint(&mut self, to: AccountId, mint_amount: u64) -> Result<(), PSP34Error>;

    /// Mint next available token for the caller
    #[ink(message, payable)]
    fn mint_next(&mut self) -> Result<(), PSP34Error>;

    #[ink(message)]
    fn refund(&mut self, token_id: u64) -> Result<(), PSP34Error>;

    // Get refund amount for given token_id
    #[ink(message)]
    fn get_refund_amount(&self, token_id: u64) -> Balance;

    /// Withdraw funds to contract owner
    #[ink(message)]
    fn withdraw_launchpad(&mut self) -> Result<(), PSP34Error>;

    /// Withdraw funds to launchpad project
    #[ink(message)]
    fn withdraw_project(&mut self) -> Result<(), PSP34Error>;

    /// Set max number of tokens which could be minted per call
    #[ink(message)]
    fn set_max_mint_amount(&mut self, max_amount: u64) -> Result<(), PSP34Error>;

    /// Get max supply of tokens
    #[ink(message)]
    fn max_supply(&self) -> u64;

    /// Get token price
    #[ink(message)]
    fn price(&self) -> Balance;

    /// Get max number of tokens which could be minted per call
    #[ink(message)]
    fn get_max_mint_amount(&mut self) -> u64;

    #[ink(message)]
    fn add_whitelisted_account_to_prepresale(
        &mut self,
        account_id: AccountId,
        mint_amount: u64,
    ) -> Result<(), PSP34Error>;

    #[ink(message)]
    fn add_whitelisted_account_to_presale(
        &mut self,
        account_id: AccountId,
        mint_amount: u64,
    ) -> Result<(), PSP34Error>;

    #[ink(message)]
    fn set_minting_status(&mut self, minting_status_index: Option<u8>) -> Result<(), PSP34Error>;

    #[ink(message)]
    fn get_minting_status(&self) -> String;
}
