use anchor_lang::prelude::*;
use mpl_core::{
    accounts::BaseCollectionV1, fetch_plugin, instructions::{AddCollectionPluginV1CpiBuilder, UpdateCollectionPluginV1CpiBuilder}, types::{Attribute, Attributes, Plugin, PluginAuthority, PluginType}
};
use crate::error::ErrorCode;

pub fn update_total_staked<'info>(
    collection: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    update_authority: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    mpl_core_program: &AccountInfo<'info>,
    signer_seeds: &[&[u8]],
    delta: i64,
) -> Result<()> {
    let attributes_fetched:Option<Attributes> = fetch_plugin::<BaseCollectionV1, Attributes>(
        collection,
        PluginType::Attributes
    )
    .ok()
    .map(|(_,attr,_)| attr);

    let mut attributes_list: Vec<Attribute> = Vec::new();
    let mut total_staked: u64 = 0;

    if let Some(attributes) = &attributes_fetched {
        for attribute in &attributes.attribute_list {
            if attribute.key == "total_staked" {
                total_staked = attribute.value.parse::<u64>().map_err(|_| ErrorCode::InvalidTotalStaked)?;
            } else {
                attributes_list.push(attribute.clone());
            }
        }
    }

    total_staked = total_staked
    .checked_add_signed(delta)
    .ok_or(ErrorCode::InvalidTotalStaked)?;

    attributes_list.push(Attribute { key: "total_staked".to_string(), value: total_staked.to_string() });

    if attributes_fetched.is_none() {
        AddCollectionPluginV1CpiBuilder::new(mpl_core_program)
        .collection(collection)
        .payer(payer)
        .authority(Some(update_authority))
        .system_program(system_program)
        .plugin(Plugin::Attributes(Attributes { attribute_list: attributes_list }))
        .init_authority(PluginAuthority::UpdateAuthority)
        .invoke_signed(&[signer_seeds])?;
    }else {
        UpdateCollectionPluginV1CpiBuilder::new(mpl_core_program)
        .collection(collection)
        .payer(payer)
        .authority(Some(update_authority))
        .system_program(system_program)
        .plugin(Plugin::Attributes(Attributes { attribute_list: attributes_list }))
        .invoke_signed(&[signer_seeds])?;
    }

    Ok(())
}
