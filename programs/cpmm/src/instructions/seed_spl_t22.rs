use integer_sqrt::IntegerSquareRoot;

use anchor_lang::prelude::*;
use anchor_spl::token::{burn, Burn, Token};

use crate::{initialize_cpmm, Initialize};

pub const RAYDIUM_FEE: u64 = 400000000; // 0.4 sol

#[derive(Accounts)]
struct BurnToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: pool lp mint, init by cp-swap
    #[account(
        mut,
        mint::token_program = token_program,
    )]
    pub mint_account: UncheckedAccount<'info>,

    /// CHECK: creator lp ATA token account, init by cp-swap
    #[account(mut)]
    pub token_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn seed_spl_t22(ctx: Context<Initialize>, init_amount_0: u64, init_amount_1: u64, open_time: u64) -> Result<()> {
    msg!("Running raydium cp swap initialize");

    let _ = initialize_cpmm(&ctx, init_amount_0, init_amount_1, open_time);

    msg!("Running lp burn from pool creator");

    let liquidity: u64 = u128::from(init_amount_0)
        .checked_mul(init_amount_1.into())
        .unwrap()
        .integer_sqrt()
        .try_into()
        .unwrap();
    let lock_lp_amount = 100;

    burn_lp(
        Context {
            program_id: &Token::id(),
            accounts: &mut BurnToken {
                authority: ctx.accounts.creator.clone(),
                mint_account: ctx.accounts.lp_mint.clone(),
                token_account: ctx.accounts.creator_lp_token.clone(),
                token_program: ctx.accounts.token_program.clone(),
            },
            remaining_accounts: &[],
            bumps: BurnTokenBumps {},
        },
        liquidity.checked_sub(lock_lp_amount).unwrap(),
    )
}

fn burn_lp(ctx: Context<BurnToken>, burn_amount: u64) -> Result<()> {
    let cpi_accounts = Burn {
        authority: ctx.accounts.authority.to_account_info(),
        from: ctx.accounts.token_account.to_account_info(),
        mint: ctx.accounts.mint_account.to_account_info(),
    };
    let cpi_context = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);

    msg!("{} LP will be burned...", burn_amount);

    burn(cpi_context, burn_amount)
}
