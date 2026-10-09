use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Only the counter authority can update this counter")]
    Unauthorized,
    #[msg("Counter has reached the maximum value")]
    CounterOverflow,
    #[msg("Invalid owner")]
    InvalidOwner,
    #[msg("Invalid update authority")]
    InvalidUpdateAuthority,
    #[msg("Already staked")]
    AlreadyStaked,
    #[msg("Asset not staked")]
    AssetNotStaked,
    #[msg("Invalid timestamp")]
    InvalidTimeStamp,
    #[msg("Freeze period not elapsed")]
    FreezePeriodNotElapsed,
    #[msg("Invalid reward bps")]
    InvalidRewardsBps,
    #[msg("Invalid total staked count")]
    InvalidTotalStaked
}
