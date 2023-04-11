use ink::prelude::string::{String as PreludeString, ToString};

pub use crate::traits::psp34_traits::Psp34Traits;

use openbrush::{
    contracts::{
        ownable::*,
        psp34::extensions::{enumerable::*, metadata::*},
    },
    modifiers,
    traits::{Storage, String},
};

pub trait Internal {
    fn token_exists(&self, id: Id) -> Result<(), PSP34Error>;
}

impl<T> Internal for T
where
    T: Storage<psp34::Data<enumerable::Balances>>,
{
    /// Check if token is minted
    default fn token_exists(&self, id: Id) -> Result<(), PSP34Error> {
        self.data::<psp34::Data<enumerable::Balances>>()
            .owner_of(id)
            .ok_or(PSP34Error::TokenNotExists)?;
        Ok(())
    }
}

impl<T> Psp34Traits for T
where
    T: Storage<psp34::Data<enumerable::Balances>>
        + Storage<ownable::Data>
        + Storage<metadata::Data>
        + psp34::extensions::metadata::PSP34Metadata,
{
    /// Get URI from token ID
    default fn token_uri(&self, token_id: u64) -> Result<PreludeString, PSP34Error> {
        self.token_exists(Id::U64(token_id))?;
        let value = self.get_attribute(
            self.data::<psp34::Data<enumerable::Balances>>()
                .collection_id(),
            String::from("baseUri"),
        );
        let mut token_uri = PreludeString::from_utf8(value.unwrap()).unwrap();
        token_uri = token_uri + &token_id.to_string() + &PreludeString::from(".json");
        Ok(token_uri)
    }

    /// Set new value for the baseUri
    #[modifiers(only_owner)]
    default fn set_base_uri(&mut self, uri: PreludeString) -> Result<(), PSP34Error> {
        let id = self
            .data::<psp34::Data<enumerable::Balances>>()
            .collection_id();
        self.data::<metadata::Data>()
            ._set_attribute(id, String::from("baseUri"), uri.into_bytes());
        Ok(())
    }
}
