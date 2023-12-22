use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_lang::solana_program::system_program;
use anchor_lang::solana_program::program::{invoke, invoke_signed};

use std::mem;
use std::str::FromStr;
use std::collections::HashSet;

pub mod contexts;
pub mod states;

pub use contexts::*;
pub use states::*;


declare_id!("5GUctGGG8KFkoS5LbQSdfGuDGeFd3jkkcVZVfHzBB6hF");

#[program]
pub mod pricelocker {

    use super::*;

    /// Create new price feed account to track price of a currency
    pub fn create_pricefeed_account(ctx: Context<CreatePricefeedAccount>, pricefeed_alias: String, pricefeed_id_string: String) -> Result<()> {
        let alias = pricefeed_alias.clone();
        let pricefeed_id = Pubkey::from_str(&pricefeed_id_string).unwrap();
        let bump = ctx.bumps.price_feed;
        ctx.accounts.process(alias, pricefeed_id, bump)
    }

    /// Create new price locker
    pub fn create_new_locker(ctx: Context<CreateNewLocker>, _locker_name: String) -> Result<()> {
        let bump = ctx.bumps.locker;
        ctx.accounts.process(bump)
    }


    /// Deposit to price locker
    pub fn deposit_funds(ctx: Context<DepositFunds>, _locker_name: String, amount: u32) -> Result<()> {
        ctx.accounts.process(amount)
    }
    

    pub fn stake_funds(ctx: Context<StakeFunds>, _locker_name: String, amount: u32) -> Result<()> {
        ctx.accounts.process(amount)
    }


    pub fn price_lock_funds(ctx: Context<PriceLockFunds>, _locker_name: String, strike_price: u32, payout_amount: u32, token_mint: Option<Pubkey>, join: Option<u8>) -> Result<()> {

        ctx.accounts.process(strike_price, payout_amount, token_mint, join)
    }


    pub fn time_lock_funds(ctx: Context<TimeLockFunds>, _locker_name: String, strike_time: u32, payout_amount: u32, token_mint: Option<Pubkey>, join: Option<u8>) -> Result<()> {
        ctx.accounts.process(strike_time, payout_amount, token_mint, join)
    }



    // /// Withdraw from price locker
    pub fn time_unlock_funds(ctx: Context<TimeUnlockFunds>, _locker_name: String, lock_index: u8) -> Result<()> {
        ctx.accounts.process(lock_index)
    }

    pub fn price_unlock_funds(ctx: Context<PriceUnlockFunds>, _locker_name: String, lock_index: u8) -> Result<()> {
        ctx.accounts.process(lock_index)
    }

    // // TODO: decide on using the term stake or delegate
    // pub fn unstake_funds(ctx: Context<UnlockFunds>, amount: u128) -> Result<()> {
    //     ctx.accounts.process(amount)
    // }

    pub fn withdraw_unlocked_funds(ctx: Context<WithdrawUnlockedFunds>, _locker_name: String, amount: u32, token_mint: Option<Pubkey>) -> Result<()> {
        ctx.accounts.process(amount, token_mint)
    }


}

