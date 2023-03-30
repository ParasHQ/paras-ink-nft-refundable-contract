use ink_prelude::vec::Vec;

use openbrush::traits::{Balance, String};
pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

use ink_env::AccountId;
pub type MilliSeconds = u64;
pub type Percentage = u64;

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
    pub first_refund_period: MilliSeconds,
    pub first_refund_share: Percentage,
    pub second_refund_period: MilliSeconds,
    pub second_refund_share: Percentage,
    pub third_refund_period: MilliSeconds,
    pub third_refund_share: Percentage,
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
        }
    }
}
