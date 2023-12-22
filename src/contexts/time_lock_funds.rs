use crate::*;

// Add a price lock to the locker
// This will ensure the user can not access the funds before the given date (timestamp)
#[derive(Accounts)]
#[instruction(locker_name: String)]
pub struct TimeLockFunds<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"locker".as_ref(), authority.key().as_ref(), &locker_name.as_ref()],
        // Ensure that the signer is the authority/owner of the locker
        constraint = locker.authority == *authority.key,  
        bump)]
    pub locker: Account<'info, Locker>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl<'info> TimeLockFunds<'_> {
    pub fn process(&mut self, strike_time: u32, amount: u32, token_mint: Option<Pubkey>, join: Option<u8>) -> Result<()> {
        let Self {locker,..} = self;

        // Check if the payout amount is more than 0, otherwise the lock is not locking any funds
        require!((amount > 0), LockerErrorCode::PayoutAmountNotPositive);


        // Get total balance of the locker
        let mut available_balance = locker.get_lamports();

        // Iterator through locks to subtract the locked amounts from the available balance
        // When user has 0 locks yet, the available balance will equal the total balance
        // With every lock, funds are locked and not available for a new lock
        for lock_item in &locker.locks {  
            // Do checks and retrieve amount of funds locked
            let lock_item_balance = get_time_locked_balance(lock_item, &token_mint);
            // Subtract the funds locked from the available balance
            available_balance -= lock_item_balance;
        }

        // Check if the amount the user wants to lock is equal or lower than the available balance
        require!(((amount as u64) <= available_balance), LockerErrorCode::PayoutAmountExceedsAvailableBalance);


        // The lock id is equivalent to the position in the vector 
        let lock_id: u8 = locker.locks.len() as u8;

        // Construct the new price lock object 
        let new_time_lock = Lock::TimeLock{
            id: lock_id,
            strike_time: strike_time,
            amount: amount,
            token_mint: token_mint,
            locked: true,
            join: join,
        };

        // Add price lock to locker vector
        locker.locks.push(new_time_lock);
        

        // Update the locked and unlocked balance
        // locker.unlocked_balance -= payout_amount;
        locker.locked_balance += amount;


        Ok(())
    }
}


// Perform checks and get the locked balance from a lock
fn get_time_locked_balance<'info>(lock_item: &Lock, token_mint_user: &Option<Pubkey>) -> u64 {

    // Check if lock is a price lock, and if so access the values 
    if let Lock::TimeLock { id, strike_time, amount, token_mint, locked, join } = lock_item {
        
        // Check if the lock is locking up tokens that we are looking for
        if &token_mint_user.unwrap() == &token_mint.unwrap() &&
            // Check if the lock is locked
            *locked == true
            {
                // Lock is locked, retrieve the locked balance
                let locked_balance = *amount as u64;
                return locked_balance

        } else {
            // the price lock is not for the token mint we are looking for - or lock is unlocked
            return 0;
        }
    } else {
        // Lock is not a Price lock
        // Adding >2 price locks on the same balance is not allowed, but combining a price lock and a time lock is allowed
        return 0;
    }
}