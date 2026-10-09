pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use state::*;
pub use instructions::*;

declare_id!("GR2w32VfkaJAcfrNXgmDTNTi1jmYjXRA6Evnz8nUWkcT");

#[program]
pub mod week5_assgn {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>, rewards_bps: u16, freeze_period: u16) -> Result<()>{
        ctx.accounts.initialize(rewards_bps, freeze_period, ctx.bumps)
    }
    pub fn create_collection(ctx: Context<CreateCollection>, name: String, uri: String) -> Result<()> {
        ctx.accounts.create(name, uri, ctx.bumps)
    }
    pub fn mint_assets(ctx: Context<MintAsset>, name: String, uri: String) -> Result<()> {
        ctx.accounts.create(name, uri, ctx.bumps)
    }
    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        ctx.accounts.stake(ctx.bumps)
    }
    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        ctx.accounts.unstake(ctx.bumps)
    }
    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        ctx.accounts.claim_rewards(ctx.bumps)
    }
    pub fn burn_staked_nft(ctx: Context<BurnStakedNft>) -> Result<()> {
        ctx.accounts.burn_staked_nft(ctx.bumps)
    }

}
