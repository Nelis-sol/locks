use crate::*;
use anchor_lang::solana_program::stake::instruction as stake_instruction;
use anchor_lang::solana_program::{stake, system_instruction, pubkey::Pubkey};



#[derive(Accounts)]
#[instruction(locker_name: String, lamports: u64)]
pub struct CreateAndDelegateStake<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        seeds = [b"stake_account", payer.key().as_ref(), locker_name.as_bytes()],
        bump,
        space = 8 + stake::state::StakeStateV2::size_of()
    )]
    /// CHECK: todo
    pub stake_account: AccountInfo<'info>,
    #[account(mut,
        seeds = [b"locker", authority.key().as_ref(), locker_name.as_bytes()],
        bump,
        constraint = locker.authority == *authority.key
    )]
    pub locker: Account<'info, Locker>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: todo
    pub vote_account: AccountInfo<'info>, // Non-mutable
    pub clock: Sysvar<'info, Clock>,
    pub stake_history: Sysvar<'info, StakeHistory>,
    /// CHECK: todo
    pub stake_config: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl<'info> CreateAndDelegateStake<'info> {
    pub fn process(&mut self, locker_name: String, lamports: u64) -> Result<()> {
        // Check if the stake account is rent-exempt
        let rent = &self.rent;
        if !rent.is_exempt(self.stake_account.lamports(), self.stake_account.data_len()) {
            return Err(ProgramError::AccountNotRentExempt.into());
        }

        // Create a new stake account
        let create_account_ix = system_instruction::create_account(
            &self.payer.key,
            &self.stake_account.key,
            lamports,
            stake::state::StakeStateV2::size_of() as u64,
            &stake::program::id(),
        );

        // Delegate the stake
        let delegate_stake_ix = stake_instruction::delegate_stake(
            &self.stake_account.key,
            &self.locker.key(),
            &self.vote_account.key,
        );

        // Send instructions
        anchor_lang::solana_program::program::invoke(
            &create_account_ix,
            &[
                self.payer.to_account_info(),
                self.stake_account.to_account_info(),
                self.system_program.to_account_info(),
            ],
        )?;

        anchor_lang::solana_program::program::invoke(
            &delegate_stake_ix,
            &[
                self.stake_account.to_account_info(),
                self.clock.to_account_info(),
                self.stake_history.to_account_info(),
                self.locker.to_account_info(),
                self.vote_account.to_account_info(),
                self.stake_config.to_account_info(),
            ],
        )?;

        Ok(())
    }
}
