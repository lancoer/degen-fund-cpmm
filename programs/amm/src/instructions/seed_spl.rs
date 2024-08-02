use anchor_lang::prelude::*;

use anchor_spl::{
    token_interface::{ TokenAccount, Mint, TransferChecked, transfer_checked },
    associated_token::AssociatedToken,
    token::Token,
};

use anchor_lang::solana_program::program::invoke_signed;
use crate::{ curve::Curve, states::*, error::ErrorCode, dex_instructions };

pub const RAYDIUM_FEE: u64 = 400000000; // 0.4 sol

#[derive(Accounts)]
pub struct InitializeLp<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub base_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        seeds = [GLOBAL_SEED.as_bytes()],
        bump,
    )]
    pub global: Box<Account<'info, Global>>,
    #[account(
        mut,
        seeds = [QUOTE_CONFIG_SEED.as_bytes(), quote_mint.key().as_ref()],
        bump,
    )]
    pub quote_config: Box<Account<'info, QuoteConfig>>,
    #[account(
        mut,
        seeds = [POOL_SEED.as_bytes(), base_mint.key().as_ref()],
        bump,
    )]
    pub pool: Box<Account<'info, Pool>>,
    #[account(
        mut,
        token::mint = base_mint,
        token::authority = pool,
        seeds = [POOL_VAULT_SEED.as_bytes(), pool.key().as_ref(), base_mint.key().as_ref()],
        bump
    )]
    pub base_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        token::mint = quote_mint,
        token::authority = pool,
        seeds = [POOL_VAULT_SEED.as_bytes(), pool.key().as_ref(), quote_mint.key().as_ref()],
        bump
    )]
    pub quote_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [FEE_COLLECTOR_SEED.as_bytes()],
        bump,
    )]
    pub fee_collector: Box<Account<'info, FeeCollector>>,
    #[account(
        mut,
        associated_token::mint = quote_mint,
        associated_token::authority = fee_collector,
    )]
    pub fee_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: Safe
    pub amm_program: AccountInfo<'info>,
    /// CHECK: Safe
    #[account(
        mut,
        seeds = [
            amm_program.key.as_ref(),
            serum_market.key.as_ref(),
            b"amm_associated_seed"],
        bump,
        seeds::program = amm_program.key
    )]
    pub amm: AccountInfo<'info>,
    /// CHECK: Safe
    #[account(seeds = [b"amm_config_account_seed"], bump, seeds::program = amm_program.key)]
    pub amm_config: AccountInfo<'info>,
    /// CHECK: Safe
    #[account(seeds = [b"amm authority"], bump, seeds::program = amm_program.key)]
    pub amm_authority: AccountInfo<'info>,
    /// CHECK: Safe
    #[account(
        mut,
        seeds = [
            amm_program.key.as_ref(),
            serum_market.key.as_ref(),
            b"open_order_associated_seed"],
        bump,
        seeds::program = amm_program.key
    )]
    pub amm_open_orders: AccountInfo<'info>,
    /// CHECK: Safe
    #[account(
        mut,
        seeds = [
            amm_program.key.as_ref(),
            serum_market.key.as_ref(),
            b"lp_mint_associated_seed"
        ],
        bump,
        seeds::program = amm_program.key
    )]
    pub lp_mint: AccountInfo<'info>,

    /// CHECK: Safe
    #[account(
        mut,
        seeds = [
            amm_program.key.as_ref(),
            serum_market.key.as_ref(),
            b"coin_vault_associated_seed"
        ],
        bump,
        seeds::program = amm_program.key
    )]
    pub pool_coin_token_account: AccountInfo<'info>,
    /// CHECK: Safe
    #[account(
        mut,
        seeds = [
            amm_program.key.as_ref(),
            serum_market.key.as_ref(),
            b"pc_vault_associated_seed"
        ],
        bump,
        seeds::program = amm_program.key
    )]
    pub pool_pc_token_account: AccountInfo<'info>,
    /// CHECK: Safe
    #[account(
        mut,
        seeds = [
            amm_program.key.as_ref(),
            serum_market.key.as_ref(),
            b"target_associated_seed"
        ],
        bump,
        seeds::program = amm_program.key
    )]
    pub amm_target_orders: AccountInfo<'info>,
    /// CHECK: Safe
    #[account(mut)]
    pub fee_destination: AccountInfo<'info>,
    /// CHECK: Safe
    #[account(
        mut,
        seeds = [
            amm_program.key.as_ref(),
            serum_market.key.as_ref(),
            b"temp_lp_token_associated_seed"
        ],
        bump,
        seeds::program = amm_program.key
    )]
    pub pool_temp_lp: AccountInfo<'info>,

    #[account(mut)]
    /// CHECK: Safe
    pub lp_vault: AccountInfo<'info>,
    /// CHECK: Safe
    pub serum_program: AccountInfo<'info>,
    /// CHECK: Safe
    pub serum_market: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn seed_spl(ctx: Context<InitializeLp>, nonce: u8) -> Result<()> {
    let quote_config = &ctx.accounts.quote_config;
    let pool = &mut ctx.accounts.pool;

    // let base_mint = &ctx.accounts.base_mint;
    let quote_mint = &ctx.accounts.quote_mint;

    // let base_vault = &ctx.accounts.base_vault;
    let quote_vault = &ctx.accounts.quote_vault;
    let fee_vault = &ctx.accounts.fee_vault;

    require!(pool.is_filled, ErrorCode::TradingNotEnded);

    let base_mint_id = ctx.accounts.base_mint.to_account_info().key();
    let pool_seed = &[POOL_SEED.as_bytes(), base_mint_id.as_ref(), &[ctx.bumps.pool]];
    let pool_signer = [&pool_seed[..]];

    let base_reserve = ctx.accounts.base_vault.amount;
    let mut quote_reserve = ctx.accounts.quote_vault.amount;

    let seed_fee = Curve::seeding_fee(quote_reserve as u128, quote_config.seeding_fee_bps);
    quote_reserve -= seed_fee as u64;

    // transfer fee amount to fee vault
    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                mint: quote_mint.to_account_info(),
                from: quote_vault.to_account_info(),
                to: fee_vault.to_account_info(),
                authority: pool.to_account_info(),
            },
            &pool_signer
        ),
        seed_fee as u64,
        quote_mint.decimals
    )?;

    let opentime = Clock::get()?.unix_timestamp as u64;
    let coin_amount: u64 = base_reserve;
    let pc_amount: u64 = quote_reserve;

    msg!("Running raydium amm initialize2");
    let initialize_ix = dex_instructions::initialize_amm(
        ctx.accounts.amm_program.key,
        ctx.accounts.amm.key,
        ctx.accounts.amm_authority.key,
        ctx.accounts.amm_open_orders.key,
        ctx.accounts.lp_mint.key,
        ctx.accounts.quote_mint.to_account_info().key,
        ctx.accounts.base_mint.to_account_info().key,
        ctx.accounts.pool_coin_token_account.key,
        ctx.accounts.pool_pc_token_account.key,
        ctx.accounts.amm_target_orders.key,
        ctx.accounts.amm_config.key,
        ctx.accounts.fee_destination.key,
        ctx.accounts.serum_program.key,
        ctx.accounts.serum_market.key,
        pool.to_account_info().key,
        ctx.accounts.base_vault.to_account_info().key,
        ctx.accounts.quote_vault.to_account_info().key,
        &ctx.accounts.lp_vault.key(),
        nonce,
        opentime,
        pc_amount,
        coin_amount
    )?;
    let account_infos = [
        ctx.accounts.amm_program.clone(),
        ctx.accounts.amm.clone(),
        ctx.accounts.amm_authority.clone(),
        ctx.accounts.amm_open_orders.clone(),
        ctx.accounts.lp_mint.clone(),
        ctx.accounts.base_mint.to_account_info(),
        ctx.accounts.quote_mint.to_account_info(),
        ctx.accounts.pool_coin_token_account.clone(),
        ctx.accounts.pool_pc_token_account.clone(),
        ctx.accounts.amm_target_orders.clone(),
        ctx.accounts.amm_config.clone(),
        ctx.accounts.fee_destination.clone(),
        ctx.accounts.serum_program.clone(),
        ctx.accounts.serum_market.clone(),
        pool.to_account_info().clone(),
        ctx.accounts.base_vault.to_account_info(),
        ctx.accounts.quote_vault.to_account_info(),
        ctx.accounts.lp_vault.clone(),
        ctx.accounts.token_program.to_account_info().clone(),
        ctx.accounts.system_program.to_account_info().clone(),
        ctx.accounts.associated_token_program.to_account_info().clone(),
        ctx.accounts.rent.to_account_info().clone(),
    ];

    invoke_signed(&initialize_ix, &account_infos, &pool_signer)?;

    pool.is_seeded = true;
    pool.base_reserve = 0;
    pool.quote_reserve = 0;

    Ok(())
}
