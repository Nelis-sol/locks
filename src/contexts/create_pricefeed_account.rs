use crate::*;


// Lockers are differentiated by a locker_name, so 1 user can have multiple lockers
// One locker can have multiple locks (e.g. 1 time lock and 1 price lock, max 10 locks), 
//  so there can be multiple conditions (AND/OR) for unlocking user funds
#[derive(Accounts)]
#[instruction(pricefeed_alias: String, pricefeed_input: String)]
pub struct CreatePricefeedAccount<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, 
        payer = authority, 
        // TODO: 100 is hardcoded, determine the size of 1 lock and set a max of locks (e.g. 10)
        space = 8 + mem::size_of::<Locker>() + 100, 
        seeds = [b"pricefeed".as_ref(), &pricefeed_alias.as_ref()], 
        bump)]
    pub price_feed: Account<'info, Pricefeedaccount>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl<'info> CreatePricefeedAccount<'_> {
    pub fn process(&mut self, pricefeed_alias: String, pricefeed_id: Pubkey, bump: u8) -> Result<()> {
        let Self {price_feed,..} = self;

        price_feed.pricefeed_alias = pricefeed_alias;

        // set signer as authority
        price_feed.pricefeed_id = pricefeed_id;

        price_feed.bump = bump;

        Ok(())

    }
}