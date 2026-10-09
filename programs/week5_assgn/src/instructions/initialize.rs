use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenInterface, Mint};
use mpl_core::accounts::BaseCollectionV1;
use crate::{Config, error::ErrorCode};

#[derive(Accounts)]
pub struct Initialize <'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = Config::DISCRIMINATOR.len() + Config::INIT_SPACE,
        seeds = [b"config", collection.key().as_ref()],
        bump
    )]
    pub config: Account<'info, Config>,
    #[account(has_one = update_authority @  ErrorCode::InvalidUpdateAuthority )]
    pub collection: Account<'info, BaseCollectionV1>,
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    /// CHECK: The account is not initialized and only used for signing purposes
    pub update_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = admin,
        mint::decimals = 6,
        mint::authority = config,
        seeds = [b"rewards_mint", config.key().as_ref()],
        bump
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>
}

impl <'info> Initialize <'info> {
        pub fn initialize(&mut self, rewards_bps: u16, freeze_period: u16, bumps: InitializeBumps) -> Result<()>{
        self.config.set_inner(Config { 
            rewards_bps, 
            freeze_period, 
            rewards_bumps: bumps.rewards_mint, 
            bumps: bumps.config
        });
        Ok(())
    }
}