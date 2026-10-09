use anchor_lang::{prelude::*};
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, mint_to_checked, MintToChecked}};
use mpl_core::{
    ID as MPL_CORE_ID, accounts::{BaseAssetV1, BaseCollectionV1}, fetch_plugin, instructions::{AddPluginV1CpiBuilder, BurnV1CpiBuilder, UpdatePluginV1CpiBuilder}, types::{Attributes, BurnDelegate, FreezeDelegate, Plugin, PluginAuthority, PluginType, UpdateAuthority}
};
use crate::{Config, error::ErrorCode, instructions::update_total_staked};
const SECONDS_PER_DAY: i64 = 86400;
// One-time bonus (in whole reward tokens) paid on top of the staking rewards for burning the NFT.
const BURN_BONUS: u64 = 1000;

#[derive(Accounts)]
pub struct BurnStakedNft <'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        seeds = [b"config", collection.key().as_ref()],
        bump = config.bumps
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        has_one = update_authority @ ErrorCode::InvalidUpdateAuthority
    )]
    pub collection: Account<'info, BaseCollectionV1>,
    #[account(
        mut,
        has_one = owner @ ErrorCode::InvalidOwner,
        constraint = asset.update_authority == UpdateAuthority::Collection(collection.key()) @ ErrorCode::InvalidUpdateAuthority,
    )]
    pub asset: Account<'info, BaseAssetV1>,
    /// CHECK: This account is not initialized and is being used for signing purposes only, we verify that derives from the correct seeds
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,
        #[account(
        mut,
        seeds = [b"rewards_mint", config.key().as_ref()],
        bump = config.rewards_bumps,
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,
       #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = rewards_mint,
        associated_token::authority = owner,
    )]
    pub user_rewards_ata: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is the ID of the MPL Core Program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>
}

impl <'info> BurnStakedNft <'info> {
    pub fn burn_staked_nft(&mut self, bumps: BurnStakedNftBumps)->Result<()>{
        let fetched_attributes:Option<Attributes> = fetch_plugin::<BaseAssetV1, Attributes>(
            &self.asset.to_account_info(),
            PluginType::Attributes
        )
        .ok()
        .map(|(_,attr,_)| attr);

        require!(fetched_attributes.is_some(), ErrorCode::AssetNotStaked);
        let attributes = fetched_attributes.unwrap();

        let current_timestamp = Clock::get()?.unix_timestamp;
        let mut is_staked = false;
        let mut staked_timestamp: i64 = 0;
        let mut staked_time: i64 = 0;

        for attr in attributes.attribute_list {
            if attr.key == "staked" {
                is_staked = attr.value == "true";
            }
            else if  attr.key == "staked_at" {
                staked_timestamp = staked_timestamp
                .checked_add(attr.value.parse::<i64>().map_err(|_| ErrorCode::InvalidTimeStamp)?)
                .ok_or(ErrorCode::InvalidTimeStamp)?;

                staked_time = current_timestamp
                .checked_sub(staked_timestamp)
                .ok_or(ErrorCode::InvalidTimeStamp)?;

                staked_time = staked_time
                .checked_div(SECONDS_PER_DAY)
                .ok_or(ErrorCode::InvalidTimeStamp)?;
            }
        }

        require!(is_staked, ErrorCode::AssetNotStaked);

        let collection_key = self.collection.key();
        let signer_seeds = &[
            b"update_authority",
            collection_key.as_ref(),
            &[bumps.update_authority],
        ];

        UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
        .asset(&self.asset.to_account_info())
        .collection(Some(&self.collection.to_account_info()))
        .payer(&self.owner.to_account_info())
        .authority(Some(&self.update_authority.to_account_info()))
        .system_program(&self.system_program.to_account_info())
        .plugin(Plugin::FreezeDelegate(FreezeDelegate {frozen: false}))
        .invoke_signed(&[signer_seeds])?;

        // BurnDelegate is owner managed, so the owner adds it and delegates burning to the update authority PDA.
        AddPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
        .asset(&self.asset.to_account_info())
        .collection(Some(&self.collection.to_account_info()))
        .payer(&self.owner.to_account_info())
        .authority(Some(&self.owner.to_account_info()))
        .system_program(&self.system_program.to_account_info())
        .plugin(Plugin::BurnDelegate(BurnDelegate {}))
        .init_authority(PluginAuthority::UpdateAuthority)
        .invoke()?;

        BurnV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
        .asset(&self.asset.to_account_info())
        .collection(Some(&self.collection.to_account_info()))
        .payer(&self.owner.to_account_info())
        .authority(Some(&self.update_authority.to_account_info()))
        .system_program(Some(&self.system_program.to_account_info()))
        .invoke_signed(&[signer_seeds])?;

        update_total_staked(
            &self.collection.to_account_info(),
            &self.owner.to_account_info(),
            &self.update_authority.to_account_info(),
            &self.system_program.to_account_info(),
            &self.mpl_core_program.to_account_info(),
            signer_seeds,
            -1,
        )?;

        let staking_rewards = (staked_time as u64)
        .checked_mul(self.config.rewards_bps as u64)
        .ok_or(ErrorCode::InvalidRewardsBps)?
        .checked_mul(10u64.pow(self.rewards_mint.decimals as u32))
        .ok_or(ErrorCode::InvalidRewardsBps)?
        .checked_div(10000u64)
        .ok_or(ErrorCode::InvalidRewardsBps)?;

        let amount = BURN_BONUS
        .checked_mul(10u64.pow(self.rewards_mint.decimals as u32))
        .and_then(|bonus| bonus.checked_add(staking_rewards))
        .ok_or(ErrorCode::InvalidRewardsBps)?;

        let config_signer_seeds: &[&[&[u8]]] = &[
            &[
                b"config",
                collection_key.as_ref(),
                &[self.config.bumps]
            ]
        ];

        let ctx_accounts = MintToChecked {
            mint: self.rewards_mint.to_account_info(),
            to: self.user_rewards_ata.to_account_info(),
            authority: self.config.to_account_info()
        };
        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            ctx_accounts,
            config_signer_seeds
        );
        mint_to_checked(ctx, amount, self.rewards_mint.decimals)?;

        Ok(())
    }
}
