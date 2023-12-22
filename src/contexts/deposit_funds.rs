use crate::*;

use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use anchor_spl::associated_token;


#[derive(Accounts)]
#[instruction(locker_name: String, amount: u64)]
pub struct DepositFunds<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut,
        seeds = [b"locker".as_ref(), authority.key().as_ref(), &locker_name.as_ref()],
        constraint = locker.authority == *authority.key, 
        bump)]
    pub locker: Account<'info, Locker>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
    /// Conditional accounts for SPL token deposit
    #[account()]
    pub token_mint_account_optional: Option<Account<'info, Mint>>,
    #[account(mut, constraint = token_account_optional.to_account_info().owner == &token::ID)]
    pub token_account_optional: Option<Account<'info, TokenAccount>>,
    #[account(address = token::ID)]
    pub token_program_optional: Option<Program<'info, Token>>,
}


impl<'info> DepositFunds<'_> {
    pub fn process(&mut self, amount: u64,) -> Result<()> {
        let Self {authority, locker, system_program, token_account_optional, token_mint_account_optional, token_program_optional,..} = self;

        // Check if the deposit is a $SOL or SPL token deposit
        // if the token_mint is None we assume a SOL deposit, otherwise SPL token deposit
        if let Some(token_mint_account) = token_mint_account_optional {


            // Unwrap the optional accounts which must contain addresses needed for the associated token account
            let token_account = token_account_optional.as_ref().unwrap();
            let token_program = token_program_optional.as_ref().unwrap();

             
            // Create new associated token account with the locker PDA as authority
            // Anchor does this 'idempotent' if it already exists it doesn't waste compute
            let cpi_accounts_create = associated_token::Create {
                payer: authority.to_account_info(),
                associated_token: token_account.to_account_info(),
                authority: locker.to_account_info(),
                mint: token_mint_account.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
            };
            associated_token::create(CpiContext::new(token_program.to_account_info(), cpi_accounts_create))?;


            // Transfer the deposit from the authority to the token PDA
            let cpi_accounts_transfer = Transfer {
                from: authority.to_account_info(),
                to: token_account.to_account_info(),
                authority: authority.to_account_info(),
            };
            let cpi_context = CpiContext::new(token_program.to_account_info(), cpi_accounts_transfer);
            token::transfer(cpi_context, amount)?;


        } else {
            
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

        }

        Ok(())
    }
}


