use crate::*;


// Lockers are differentiated by a locker_name, so 1 user can have multiple lockers
// One locker can have multiple locks (e.g. 1 time lock and 1 price lock, max 10 locks), 
//  so there can be multiple conditions (AND/OR) for unlocking user funds
#[derive(Accounts)]
#[instruction(locker_name: String)]
pub struct CreateNewLocker<'info> {
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

impl<'info> CreateNewLocker<'_> {
    pub fn process(&mut self, bump: u8) -> Result<()> {
        let Self {authority, locker,..} = self;

        // set signer as authority
        locker.authority = authority.key();

        let clock: Clock = Clock::get().unwrap();
        locker.creation_ts = clock.unix_timestamp as u32;

        // unlocked_balance starts at 0 as there is no funds deposited yet
        // locker.unlocked_balance = 0;
        locker.locked_balance = 0;

        // no locks are added yet, and no funds are staked yet
        locker.staked = false;
        
        locker.bump = bump;

        Ok(())

    }
}