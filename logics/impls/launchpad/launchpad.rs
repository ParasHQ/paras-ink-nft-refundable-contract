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
        psp34::{
            balances::BalancesManager,
            extensions::{enumerable::*, metadata::*},
        },
        reentrancy_guard::*,
    },
    modifiers,
    traits::{AccountId, Balance, Storage, String},
};

use ink_env::{hash, hash_bytes};

pub enum MintingStatus {
    Closed,
    Prepresale,
    Presale,
    Public,
    End,
}

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

    fn get_refund_amount_internal(&self, token_id: u64) -> Balance;

    fn check_allowed_to_mint(
        &mut self,
        account_id: AccountId,
        mint_amount: u64,
    ) -> Result<(), PSP34Error>;

    fn get_current_minting_status(&self) -> MintingStatus;
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
        let caller_id = Self::env().caller();
        self.check_amount(mint_amount)?;
        self.check_value(Self::env().transferred_value(), mint_amount)?;
        self.check_allowed_to_mint(caller_id, mint_amount)?;

        let mut token_ids = Vec::new();
        let current_timestamp = Self::env().block_timestamp();
        for _ in 0..mint_amount {
            let mint_id = self.get_mint_id();
            self.data::<psp34::Data<enumerable::Balances>>()
                ._mint_to(to, Id::U64(mint_id))?;
            self._emit_transfer_event(None, Some(to), Id::U64(mint_id));
            token_ids.push(mint_id);
        }

        Ok(token_ids)
    }

    /// Mint next available token for the caller
    default fn mint_next(&mut self) -> Result<u64, PSP34Error> {
        let caller_id = Self::env().caller();

        self.check_amount(1)?;
        self.check_value(Self::env().transferred_value(), 1)?;
        self.check_allowed_to_mint(caller_id, 1);

        let mint_id = self.get_mint_id();
        self.data::<psp34::Data<enumerable::Balances>>()
            ._mint_to(caller_id, Id::U64(mint_id))?;

        self._emit_transfer_event(None, Some(caller_id), Id::U64(mint_id));

        let current_timestamp = Self::env().block_timestamp();
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
        let caller_id = Self::env().caller();

        assert_eq!(caller_id, self._owner_of(&Id::U64(token_id)).unwrap()); // To Do : check if assert works

        let refund_amount = self.get_refund_amount_internal(token_id);

        if refund_amount == 0 {
            return Err(PSP34Error::Custom(String::from(
                Shiden34Error::RefundFailed.as_str(),
            )));
        } else {
            let refund_address = self.data::<Data>().refund_address;
            self._transfer_token(refund_address, Id::U64(token_id), Vec::new());
            self.data::<Data>().has_refunded.insert(token_id, &true);

            Self::env()
                .transfer(caller_id, refund_amount)
                .map_err(|_| {
                    PSP34Error::Custom(String::from(Shiden34Error::WithdrawalFailed.as_str()))
                })?;
            return Ok(());
        }
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

    default fn get_refund_amount(&self, token_id: u64) -> Balance {
        self.get_refund_amount_internal(token_id)
    }

    #[modifiers(only_owner)]
    default fn add_whitelisted_account_to_prepresale(
        &mut self,
        account_id: AccountId,
        mint_amount: u64,
    ) -> Result<(), PSP34Error> {
        self.data::<Data>()
            .prepresale_whitelisted
            .insert(account_id, &mint_amount);
        Ok(())
    }

    #[modifiers(only_owner)]
    default fn add_whitelisted_account_to_presale(
        &mut self,
        account_id: AccountId,
        mint_amount: u64,
    ) -> Result<(), PSP34Error> {
        self.data::<Data>()
            .presale_whitelisted
            .insert(account_id, &mint_amount);
        Ok(())
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
        let minting_status = self.get_current_minting_status();

        let mut price;
        match minting_status {
            MintingStatus::Prepresale => price = self.data::<Data>().prepresale_price_per_mint,
            MintingStatus::Presale => price = self.data::<Data>().presale_price_per_mint,
            MintingStatus::Public => price = self.data::<Data>().price_per_mint,
            _ => {
                return Err(PSP34Error::Custom(String::from(
                    Shiden34Error::BadMintValue.as_str(),
                )))
            }
        };

        if let Some(value) = (mint_amount as u128).checked_mul(price) {
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

    default fn check_allowed_to_mint(
        &mut self,
        account_id: AccountId,
        mint_amount: u64,
    ) -> Result<(), PSP34Error> {
        let minting_status = self.get_current_minting_status();

        match minting_status {
            MintingStatus::Closed => {
                return Err(PSP34Error::Custom(String::from(
                    Shiden34Error::UnableToMint.as_str(),
                )))
            }
            MintingStatus::End => {
                return Err(PSP34Error::Custom(String::from(
                    Shiden34Error::UnableToMint.as_str(),
                )))
            }
            MintingStatus::Prepresale => {
                let mint_slot = self
                    .data::<Data>()
                    .prepresale_whitelisted
                    .get(account_id)
                    .unwrap_or(0);

                if mint_slot < mint_amount {
                    return Err(PSP34Error::Custom(String::from(
                        Shiden34Error::UnableToMint.as_str(),
                    )));
                }
                self.data::<Data>()
                    .prepresale_whitelisted
                    .insert(account_id, &(mint_slot - mint_amount));
            }
            MintingStatus::Presale => {
                let mint_slot = self
                    .data::<Data>()
                    .presale_whitelisted
                    .get(account_id)
                    .unwrap_or(0);

                if mint_slot < mint_amount {
                    return Err(PSP34Error::Custom(String::from(
                        Shiden34Error::UnableToMint.as_str(),
                    )));
                }
                self.data::<Data>()
                    .presale_whitelisted
                    .insert(account_id, &(mint_slot - mint_amount));
            }
            MintingStatus::Public => {
                return Ok(());
            }
        }

        return Ok(());
    }

    fn get_total_available_to_withdraw(&self) -> Balance {
        return 1;
    }

    default fn get_refund_amount_internal(&self, token_id: u64) -> Balance {
        if !self
            .data::<Data>()
            .has_refunded
            .get(token_id)
            .unwrap_or(false)
        {
            return 0;
        }
        let current_timestamp = Self::env().block_timestamp();

        for (i, refund_period) in self.data::<Data>().refund_periods.iter().enumerate() {
            if current_timestamp < (self.data::<Data>().public_sale_end_at + refund_period) {
                let refund_share: Balance =
                    *self.data::<Data>().refund_shares.get(i).unwrap_or(&100);

                let refund_amount: Balance =
                    (self.data::<Data>().price_per_mint * refund_share).saturating_div(100);

                return refund_amount;
            }
        }

        return 0;
    }

    default fn get_current_minting_status(&self) -> MintingStatus {
        let current_timestamp = Self::env().block_timestamp();

        if current_timestamp > self.data::<Data>().public_sale_end_at
            || u128::from(self.data::<Data>().max_supply)
                == self
                    .data::<psp34::Data<enumerable::Balances>>()
                    .total_supply()
        {
            // or if token supply abis
            return MintingStatus::End;
        } else if current_timestamp > self.data::<Data>().public_sale_start_at {
            return MintingStatus::Public;
        } else if current_timestamp > self.data::<Data>().presale_start_at {
            return MintingStatus::Presale;
        } else if current_timestamp > self.data::<Data>().prepresale_start_at {
            return MintingStatus::Prepresale;
        } else {
            return MintingStatus::Closed;
        }
    }
}
