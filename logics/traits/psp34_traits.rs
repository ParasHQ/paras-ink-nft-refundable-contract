use ink::prelude::string::String as PreludeString;

use openbrush::contracts::psp34::PSP34Error;

#[openbrush::wrapper]
pub type Psp34TraitsRef = dyn Psp34Traits;

#[openbrush::trait_definition]
pub trait Psp34Traits {
    /// Set new value for the baseUri
    #[ink(message)]
    fn set_base_uri(&mut self, uri: PreludeString) -> Result<(), PSP34Error>;

    /// Get URI from token ID
    #[ink(message)]
    fn token_uri(&self, token_id: u64) -> PreludeString;

    /// https://github.com/Koniverse/SubWallet-Extension/blob/master/packages/extension-base/src/koni/api/nft/wasm_nft/index.ts#L88
    #[ink(message)]
    fn get_attribute_count(&self) -> u32;
}
