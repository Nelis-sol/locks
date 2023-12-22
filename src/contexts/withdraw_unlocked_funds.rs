use crate::*;

use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use anchor_spl::associated_token;


// Withdraw funds from the locker, for now only SOL - later SPL tokens are added
#[derive(Accounts)]
#[instruction(locker_name: String, amount: u64)]
pub struct WithdrawUnlockedFunds<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut,
        seeds = [b"locker".as_ref(), authority.key().as_ref(), &locker_name.as_ref()],
        constraint = locker.authority == *authority.key, 
        bump)]
    pub locker: Account<'info, Locker>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
    /// Conditional accounts for SPL token withdrawal
    #[account()]
    pub token_mint_account_optional: Option<Account<'info, Mint>>,
    #[account(mut, constraint = token_account_optional.to_account_info().owner == &token::ID)]
    pub token_account_optional: Option<Account<'info, TokenAccount>>,
    #[account(address = token::ID)]
    pub token_program_optional: Option<Program<'info, Token>>,
}

impl<'info> WithdrawUnlockedFunds<'_> {
    pub fn process(&mut self, locker_name: String, amount: u64) -> Result<()> {
        let Self { authority, locker, system_program, token_account_optional, token_mint_account_optional, token_program_optional, .. } = self;

        // assert!(amount <= locker.unlocked_balance, "Insufficient unlocked balance");
        assert!(authority.key() == locker.authority, "Signer not authorized");

        // Check if the payout amount is more than 0, otherwise the lock is not locking any funds
        require!((amount > 0), LockerErrorCode::PayoutAmountNotPositive);


        /// TODO: Only implemented available balance for $SOL, not yet for SPL tokens
         
        
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


        // Check if the deposit is a $SOL or SPL token deposit
        // if the token_mint is None we assume a SOL deposit, otherwise SPL token deposit
        if let Some(_token_mint_account) = token_mint_account_optional {
            
            // Unwrap the optional accounts which must contain addresses needed for the associated token account
            let token_account = token_account_optional.as_ref().unwrap();
            let token_program = token_program_optional.as_ref().unwrap();

            // Transfer the withdrawal amount from the token account to the authority
            let cpi_accounts = Transfer {
                from: token_account.to_account_info(),
                to: authority.to_account_info(),
                authority: locker.to_account_info(),
            };
            let cpi_context = CpiContext::new(token_program.to_account_info(), cpi_accounts);
            token::transfer(cpi_context, amount)?;
        } else {
            // Transfer SOL from the locker to the authority
            invoke_signed(
                &system_instruction::transfer(
                    &locker.to_account_info().key,
                    &authority.to_account_info().key,
                    amount,
                ),
                &[
                    locker.to_account_info().clone(),
                    authority.to_account_info().clone(),
                    system_program.to_account_info().clone(),
                ],
                &[&[b"locker".as_ref(), authority.key().as_ref(), &locker_name.as_ref(), &[locker.bump]]],
            )?;
        }
        
        Ok(())
    }
}  
