use crate::*;
use states::errors::PythErrorCode;
use states::PriceFeed;
use anchor_lang::solana_program::pubkey::Pubkey;


// Unlock funds when the strike_price (input earlier by the user) is hit
// Uses the Pyth price oracle to determine current (SOL) price
#[derive(Accounts)]
#[instruction(locker_name: String)]
pub struct PriceUnlockFunds<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub price_feed: Account<'info, Pricefeedaccount>,
    #[account(mut, seeds = [b"locker".as_ref(), authority.key().as_ref(), &locker_name.as_ref()],
        // Ensure that the signer is the authority/owner of the locker
        constraint = locker.authority == *authority.key, 
        bump)]
    pub locker: Account<'info, Locker>,
    // The public key of the Pyth SOL price feed
    // Check if the given account matches the address we statedd in our AdminConfig
    #[account(address = price_feed.pricefeed_id @ PythErrorCode::InvalidArgument)]
    pub pyth_solprice_account: Account<'info, PriceFeed>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl<'info> PriceUnlockFunds<'_> {
    pub fn process(&mut self, lock_index: u8) -> Result<()> {
        let Self {ref mut locker, pyth_solprice_account,..} = self;

        // Users can choose whether they want to unlock all unlockable funds,or just a specific one
        // Lock_index is the index used to retrieve the lock object in the locks vector
        match lock_index {
            // index 255 is code for: unlock all unlockable-locks
            255 => {

                // Loops through all available locks
                for lock_item in &mut locker.locks {

                    // Check if price lock can be openend (asset price exceeds strike price)
                    // Unlock locker if true
                    process_price_lock(lock_item, pyth_solprice_account).unwrap();
                }
            },
            // any other index than 255 leads to trying to retrieve the lock and unlock the funds
            index => {

                // Retrieves price lock from locks vector by the index
                let lock_item = locker.locks
                    .get_mut(index as usize)
                    .ok_or(LockerErrorCode::NoLockAtIndex)
                    .unwrap();

                // Check if price lock can be openend (asset price exceeds strike price)
                // Unlock locker if true
                process_price_lock(lock_item, pyth_solprice_account).unwrap();

            }
        }

        Ok(())

    }
}


// Retrieve price from Pyth pricefeed for comparison with strike price
fn get_price_from_pricefeed<'info>(pricefeed_account: &mut Account<'info, PriceFeed>) -> Result<u32> {

    // Get the current timestamp
    let current_timestamp = Clock::get()?.unix_timestamp;

    // We retrieve the price without account for confidence interval
    // More info about confidence intervals: https://docs.pyth.network/documentation/solana-price-feeds/best-practices#confidence-intervals
    let sol_price_no_conf_interval = pricefeed_account
        // make sure the stated price is max 60 seconds old
        .get_price_no_older_than(current_timestamp, 60)
        .ok_or(PythErrorCode::PythOffline)?;

    // Cast price as u32 to match the type in our Lock
    let sol_price = sol_price_no_conf_interval.price as u32;

    Ok(sol_price)

}

// Open up locks of which the current price is larger than the strike_price stated in the lock (as earlier defined by the user)
fn process_price_lock<'info>(lock_item: &mut Lock, pyth_solprice_account: &mut Account<'info, PriceFeed>) -> Result<()> {

    // Retrieve the current price from Pyth, currently this is the SOL price
    let price_from_pricefeed = get_price_from_pricefeed(pyth_solprice_account).unwrap();

    // Check if lock is a price lock, and if so access the values 
    if let Lock::PriceLock { id, strike_price, amount, token_mint, locked, join } = lock_item {
        // Check if the price of the asset exceeds the strike_price defined in the locker
        if price_from_pricefeed >= *strike_price {
            // asset price exceeds strike_price so unlock the lock
            *locked = false;
            Ok(())
        } else {
            Err(LockerErrorCode::StrikePriceTooLow.into())
        }
    } else {
        Err(LockerErrorCode::NotAPriceLock.into())
    }
}


