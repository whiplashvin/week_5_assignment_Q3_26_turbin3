use anchor_lang::prelude::*;
use mpl_core::{
    ID as MPL_CORE_ID, instructions::CreateV2CpiBuilder
};

#[derive(Accounts)]
pub struct MintAsset <'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub asset: Signer<'info>,
    pub collection: Signer<'info>,
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    /// CHECK: The account is not initialized and only used for signing purposes
    pub update_authority: UncheckedAccount<'info>,
    pub system_progam: Program<'info, System>,

    /// CHECK: This is the ID of the MPL Core Program
    #[account( address = MPL_CORE_ID )]
    pub mpl_core_program: UncheckedAccount<'info>
}

impl <'info> MintAsset <'info>  {
    pub fn create(&mut self,name: String, uri: String, bumps: MintAssetBumps) -> Result<()>{
        let collection_key = self.collection.key();
        let signer_seeds: &[&[&[u8]]] = &[
            &[
                b"update_authority",
                collection_key.as_ref(),
                &[bumps.update_authority]
            ]
        ];

        CreateV2CpiBuilder::new(&self.mpl_core_program.to_account_info())
        .asset(&self.asset.to_account_info())
        .collection(Some(&self.collection.to_account_info()))
        .payer(&self.payer.to_account_info())
        .owner(Some(&self.payer.to_account_info()))
        .authority(Some(&self.update_authority.to_account_info()))
        .update_authority(None)
        .system_program(&self.system_progam.to_account_info())
        .name(name)
        .uri(uri)
        .invoke_signed(signer_seeds)?;
        Ok(())
    }
}