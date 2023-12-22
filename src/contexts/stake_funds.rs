use crate::*;


// Stake to earn yield on locked (and unlocked) funds in the locker
#[derive(Accounts)]
#[instruction(locker_name: String)]
pub struct StakeFunds<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, 
        payer = authority, 
        // TODO: 100 is hardcoded, determine the size of 1 lock and set a max of locks (e.g. 10)
        space = 8 + mem::size_of::<Locker>() + 100, 
        seeds = [b"locker".as_ref(), authority.key().as_ref(), locker_name.as_ref()], 
        bump)]
    pub locker: Account<'info, Locker>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl<'info> StakeFunds<'_> {
    pub fn process(&mut self, amount: u32) -> Result<()> {
        let Self {authority, locker,..} = self;

        Ok(())

    }
}