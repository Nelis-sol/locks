use crate::*;

// Withdraw funds from the locker, for now only SOL - later SPL tokens are added
#[derive(Accounts)]
#[instruction(locker_name: String)]
pub struct WithdrawUnlockedFunds<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut,
        seeds = [b"locker".as_ref(), authority.key().as_ref(), locker_name.as_ref()],
        bump,
        constraint = locker.authority == *authority.key)]
    pub locker: Account<'info, Locker>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl<'info> WithdrawUnlockedFunds<'_> {
    pub fn process(&mut self, amount: u32, token_mint: Option<Pubkey>) -> Result<()> {
        let Self { authority, locker, .. } = self;

        // assert!(amount <= locker.unlocked_balance, "Insufficient unlocked balance");
        assert!(authority.key() == locker.authority, "Signer not authorized");

        // Check if the payout amount is more than 0, otherwise the lock is not locking any funds
        require!((amount > 0), LockerErrorCode::PayoutAmountNotPositive);


        // Get total balance of the locker
        let mut available_balance = locker.get_lamports();


        // Initiate locked_balance which we will fill when we find locked locks
        let mut locked_balance = 0;

        // Joined locks is a HashSet to find locks that are join together (join = id of another lock)
        let mut joined_locks = HashSet::new();


        // Iterator through locks to subtract the locked amounts from the available balance
        // When user has 0 locks yet, the available balance will equal the total balance
        // With every lock, funds are locked and not available for a new lock which will be reflected in the locked_balance variable
        for lock_item in &locker.locks { 

            match lock_item {
                // Access values of Lock items
                Lock::PriceLock { id, amount, locked, join, .. } | Lock::TimeLock { id, amount, locked, join, .. } => {
                    // Check if the lock is locked and has no dependency on other locks
                    if *locked == true && join.is_none() {
                        locked_balance += amount;
                    // Lock is locked and has a join with another lock, e.g. 100 $SOL has two locks: time lock (01-01-2025) and a price lock ($1000)
                    // We need to check if both locks are locked or if one is unlocked, in that case the funds are unlocked
                    // A user in this example basically says: unlock after 01-01-2025 OR if the $SOL price hits $1000
                    } else if *locked == true {
                        // Check if the id is already in the joined_locks
                        if !joined_locks.contains(id) {
                            // Add the id to joined_locks
                            joined_locks.insert(*id);
                        } else {
                            // The lock id is already in joined_locks
                            // This means both the locks are locked and the locked amount should be added to the locked_balance
                            locked_balance += amount;
                        }
                    }
                }
            } 
        }

        // Cast from u32 to u64
        let balance_locked = locked_balance as u64;

        // Subtract the locked amounts from the available balance
        let transfer_amount = available_balance - balance_locked;

        // Check if the amount the user wants withdraw is within the available balance
        require!((transfer_amount <= available_balance), LockerErrorCode::PayoutAmountExceedsAvailableBalance);


        // Check what token to withdraw
        if token_mint.is_none() {
            // It's a $SOL withdrawal
            // Transfer funds from locker to authority / signer
            **locker.to_account_info().try_borrow_mut_lamports()? -= transfer_amount;
            **authority.to_account_info().try_borrow_mut_lamports()? += transfer_amount;

        }
        
        Ok(())
    }
}  
