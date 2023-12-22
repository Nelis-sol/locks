use crate::*;

// Deposit funds into the locker, for now only SOL - later SPL tokens are added
#[derive(Accounts)]
#[instruction(locker_name: String)]
pub struct DepositFunds<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut,
        seeds = [b"locker".as_ref(), authority.key().as_ref(), &locker_name.as_ref()],
        // Ensure that the signer is the authority/owner of the locker
        constraint = locker.authority == *authority.key, 
        bump)]
    pub locker: Account<'info, Locker>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl<'info> DepositFunds<'_> {
    pub fn process(&mut self, amount: u32) -> Result<()> {
        let Self {authority, locker,..} = self;

        // Transfer funds from signer to the locker
        invoke(
            &system_instruction::transfer(
                // from authority
                &authority.to_account_info().key,
                // to locker
                &locker.to_account_info().key,
                // amount is a u32 type for simpler front-end integrations
                // need to convert into u64 for the transfer
                amount.into(),             
            ),
            &[
                // accounts for this transfer
                // from authority
                authority.to_account_info().clone(),
                // to locker
                locker.to_account_info().clone(),
            ],
        )?;

        // the deposited funds are unlocked (can be withdrawn at any moment)
        // add to unlocked_balance
        // locker.unlocked_balance += amount;

        Ok(())

    }

}