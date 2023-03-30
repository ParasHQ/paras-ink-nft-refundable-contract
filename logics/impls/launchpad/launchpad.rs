// Copyright (c) 2022 Astar Network
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the"Software"),
// to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use ink_prelude::string::{String as PreludeString, ToString};

use crate::impls::launchpad::types::{Data, Shiden34Error};
pub use crate::traits::launchpad::Launchpad;

use ink_prelude::vec::Vec;
use openbrush::{
    contracts::{
        ownable::*,
        psp34::extensions::{enumerable::*, metadata::*},
        reentrancy_guard::*,
    },
    modifiers,
    traits::{AccountId, Balance, Storage, String},
};

use ink_env::{hash, hash_bytes};

pub trait Internal {
    /// Check if the transferred mint values is as expected
    fn check_value(&self, transferred_value: u128, mint_amount: u64) -> Result<(), PSP34Error>;

    /// Check amount of tokens to be minted
    fn check_amount(&self, mint_amount: u64) -> Result<(), PSP34Error>;

    /// Check if token is minted
    fn token_exists(&self, id: Id) -> Result<(), PSP34Error>;

    fn get_pseudo_random(&mut self, max_amount: u64) -> u64;

    fn get_mint_id(&mut self) -> u64;

    fn get_total_available_to_withdraw(&self) -> Balance;

    fn get_refund_amount(&self) -> Balance;

    fn check_minting_available(&self) -> Result<(), PSP34Error>;
}

impl<T> Launchpad for T
where
    T: Storage<Data>
        + Storage<psp34::Data<enumerable::Balances>>
        + Storage<reentrancy_guard::Data>
        + Storage<ownable::Data>
        + Storage<metadata::Data>
        + psp34::extensions::metadata::PSP34Metadata
        + psp34::Internal,
{
    /// Mint one or more tokens
    #[modifiers(non_reentrant)]
    default fn mint(&mut self, to: AccountId, mint_amount: u64) -> Result<Vec<u64>, PSP34Error> {
        self.check_minting_available();
        self.check_amount(mint_amount)?;
        self.check_value(Self::env().transferred_value(), mint_amount)?;

        let mut token_ids = Vec::new();
        for _ in 0..mint_amount {
            let mint_id = self.get_mint_id();
            self.data::<psp34::Data<enumerable::Balances>>()
                ._mint_to(to, Id::U64(mint_id))?;
            self._emit_transfer_event(None, Some(to), Id::U64(mint_id));
            token_ids.push(mint_id)
        }

        Ok(token_ids)
    }

    /// Mint next available token for the caller
    default fn mint_next(&mut self) -> Result<u64, PSP34Error> {
        self.check_minting_available();
        self.check_amount(1)?;
        self.check_value(Self::env().transferred_value(), 1)?;
        let caller = Self::env().caller();

        let mint_id = self.get_mint_id();
        self.data::<psp34::Data<enumerable::Balances>>()
            ._mint_to(caller, Id::U64(mint_id))?;

        self._emit_transfer_event(None, Some(caller), Id::U64(mint_id));
        return Ok(mint_id);
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

    /// Withdraws funds to contract owner
    #[modifiers(only_owner)]
    default fn withdraw_launchpad(&mut self) -> Result<(), PSP34Error> {
        return Ok(());
    }

    default fn withdraw_project(&mut self) -> Result<(), PSP34Error> {
        return Ok(());
    }

    default fn refund(&mut self, token_id: u64) -> Result<(), PSP34Error> {
        return Ok(());
    }

    /// Set max number of tokens which could be minted per call
    #[modifiers(only_owner)]
    default fn set_max_mint_amount(&mut self, max_amount: u64) -> Result<(), PSP34Error> {
        self.data::<Data>().max_amount = max_amount;

        Ok(())
    }

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

    /// Get max supply of tokens
    default fn max_supply(&self) -> u64 {
        self.data::<Data>().max_supply
    }

    /// Get token price
    default fn price(&self) -> Balance {
        self.data::<Data>().price_per_mint
    }

    /// Get max number of tokens which could be minted per call
    default fn get_max_mint_amount(&mut self) -> u64 {
        self.data::<Data>().max_amount
    }
}

/// Helper trait for Launchpad
impl<T> Internal for T
where
    T: Storage<Data> + Storage<psp34::Data<enumerable::Balances>>,
{
    /// Check if the transferred mint values is as expected
    default fn check_value(
        &self,
        transferred_value: u128,
        mint_amount: u64,
    ) -> Result<(), PSP34Error> {
        if let Some(value) = (mint_amount as u128).checked_mul(self.data::<Data>().price_per_mint) {
            if transferred_value == value {
                return Ok(());
            }
        }
        return Err(PSP34Error::Custom(String::from(
            Shiden34Error::BadMintValue.as_str(),
        )));
    }

    /// Check amount of tokens to be minted
    default fn check_amount(&self, mint_amount: u64) -> Result<(), PSP34Error> {
        if mint_amount == 0 {
            return Err(PSP34Error::Custom(String::from(
                Shiden34Error::CannotMintZeroTokens.as_str(),
            )));
        }
        if mint_amount > self.data::<Data>().max_amount {
            return Err(PSP34Error::Custom(String::from(
                Shiden34Error::TooManyTokensToMint.as_str(),
            )));
        }
        let token_left = self.data::<Data>().token_set.len().clone() as u64;
        if mint_amount <= token_left {
            return Ok(());
        }
        return Err(PSP34Error::Custom(String::from(
            Shiden34Error::CollectionIsFull.as_str(),
        )));
    }

    /// Check if token is minted
    default fn token_exists(&self, id: Id) -> Result<(), PSP34Error> {
        self.data::<psp34::Data<enumerable::Balances>>()
            .owner_of(id)
            .ok_or(PSP34Error::TokenNotExists)?;
        Ok(())
    }

    default fn get_pseudo_random(&mut self, max_value: u64) -> u64 {
        let seed = Self::env().block_timestamp();
        let mut input: Vec<u8> = Vec::new();
        input.extend_from_slice(&seed.to_be_bytes());
        input.extend_from_slice(&self.data::<Data>().pseudo_random_salt.to_be_bytes());
        let mut output = <hash::Keccak256 as hash::HashOutput>::Type::default();
        hash_bytes::<hash::Keccak256>(&input, &mut output);
        self.data::<Data>().pseudo_random_salt += 1;

        // hacky, have to find another way
        let number = (output[0] as u64 * output[1] as u64) % (max_value + 1);
        number
    }

    default fn get_mint_id(&mut self) -> u64 {
        let token_length = self.data::<Data>().token_set.len().clone() as u64;
        let token_set_idx = self.get_pseudo_random(token_length - 1);
        self.data::<Data>()
            .token_set
            .swap_remove(token_set_idx as usize)
    }

    default fn check_minting_available(&self) -> Result<(), PSP34Error> {
        let time_now = Self::env().block_timestamp();
        if (time_now < self.data::<Data>().mint_start_at)
            || time_now > self.data::<Data>().mint_end_at
        {
            return Err(PSP34Error::Custom(String::from(
                Shiden34Error::NotMintingTime.as_str(),
            )));
        }
        return Ok(());
    }

    fn get_total_available_to_withdraw(&self) -> Balance {
        return 1;
    }
    fn get_refund_amount(&self) -> Balance {
        return 1;
    }
}
