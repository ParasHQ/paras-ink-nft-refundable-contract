use ink_prelude::vec::Vec;
use ink_storage::Mapping;
use openbrush::traits::{Balance, String};
pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

use ink_env::AccountId;
pub type MilliSeconds = u64;
pub type Percentage = u128;
pub type TokenId = u64;
pub type BlockTimestamp = u64;

#[derive(Default, Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub last_token_id: u64,
    pub collection_id: u32,
    pub max_supply: u64,
    pub price_per_mint: Balance,
    pub max_amount: u64,
    pub token_set: Vec<u64>,
    pub pseudo_random_salt: u64,
    pub project_account_id: AccountId,
    pub mint_start_at: u64,
    pub mint_end_at: u64,
    pub refund_periods: Vec<MilliSeconds>,
    pub refund_shares: Vec<Percentage>,
    pub minted_at: Mapping<TokenId, BlockTimestamp>,
    pub has_refunded: Mapping<TokenId, bool>,
    pub refund_address: AccountId,
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]

pub enum Shiden34Error {
    BadMintValue,
    CannotMintZeroTokens,
    CollectionIsFull,
    TooManyTokensToMint,
    WithdrawalFailed,
    NotMintingTime,
    RefundFailed,
}

impl Shiden34Error {
    pub fn as_str(&self) -> String {
        match self {
            Shiden34Error::BadMintValue => String::from("BadMintValue"),
            Shiden34Error::CannotMintZeroTokens => String::from("CannotMintZeroTokens"),
            Shiden34Error::CollectionIsFull => String::from("CollectionIsFull"),
            Shiden34Error::TooManyTokensToMint => String::from("TooManyTokensToMint"),
            Shiden34Error::WithdrawalFailed => String::from("WithdrawalFailed"),
            Shiden34Error::NotMintingTime => String::from("NotMintingTime"),
            Shiden34Error::RefundFailed => String::from("RefundFailed"),
        }
    }
}
