use anchor_lang::InstructionData;
use litesvm::{types::TransactionResult, LiteSVM};
use solana_address::Address;
use solana_clock::Clock;
use solana_instruction::{error::InstructionError, AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_signer::Signer;
use solana_transaction::Transaction;
use solana_transaction_error::TransactionError;

const SECONDS_PER_DAY: i64 = 86400;
const REWARDS_BPS: u16 = 10000;
const FREEZE_PERIOD_IN_DAYS: u16 = 7;
const TIME_TRAVEL_IN_DAYS: i64 = 8;
const REWARDS_DECIMALS: u32 = 6;

fn addr(pubkey: anchor_lang::prelude::Pubkey) -> Address {
    Address::new_from_array(pubkey.to_bytes())
}

fn program_id() -> Address {
    addr(week5_assgn::ID)
}

fn mpl_core_id() -> Address {
    addr(mpl_core::ID)
}

fn system_program_id() -> Address {
    addr(anchor_lang::system_program::ID)
}

fn token_program_id() -> Address {
    addr(anchor_spl::token::ID)
}

fn associated_token_program_id() -> Address {
    addr(anchor_spl::associated_token::ID)
}

struct Ctx {
    svm: LiteSVM,
    payer: Keypair,
    collection: Keypair,
    nft: Keypair,
    update_authority: Address,
    config: Address,
    rewards_mint: Address,
}

impl Ctx {
    fn new() -> Self {
        let mut svm = LiteSVM::new();

        let so_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/week5_assgn.so");
        svm.add_program_from_file(program_id(), so_path)
            .expect("program .so not found, run `anchor build` first");

        let core_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/mpl_core.so");
        svm.add_program_from_file(mpl_core_id(), core_path)
            .expect("mpl_core.so not found in tests/fixtures");

        let payer = Keypair::new();
        svm.airdrop(&payer.pubkey(), 100_000_000_000).unwrap();

        let collection = Keypair::new();
        let nft = Keypair::new();
        let collection_key = collection.pubkey();

        let (update_authority, _) = Address::find_program_address(
            &[b"update_authority", collection_key.as_ref()],
            &program_id(),
        );
        let (config, _) =
            Address::find_program_address(&[b"config", collection_key.as_ref()], &program_id());
        let (rewards_mint, _) =
            Address::find_program_address(&[b"rewards_mint", config.as_ref()], &program_id());

        Self { svm, payer, collection, nft, update_authority, config, rewards_mint }
    }

    fn send(&mut self, ix: Instruction, extra_signers: &[&Keypair]) -> TransactionResult {
        self.svm.expire_blockhash();
        let mut signers: Vec<&Keypair> = vec![&self.payer];
        signers.extend_from_slice(extra_signers);
        let msg = Message::new(&[ix], Some(&self.payer.pubkey()));
        let tx = Transaction::new(&signers, msg, self.svm.latest_blockhash());
        self.svm.send_transaction(tx)
    }

    fn user_rewards_ata(&self) -> Address {
        Address::find_program_address(
            &[
                self.payer.pubkey().as_ref(),
                token_program_id().as_ref(),
                self.rewards_mint.as_ref(),
            ],
            &associated_token_program_id(),
        )
        .0
    }

    fn create_collection(&mut self) -> TransactionResult {
        let ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new(self.collection.pubkey(), true),
                AccountMeta::new_readonly(self.update_authority, false),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new_readonly(mpl_core_id(), false),
            ],
            data: week5_assgn::instruction::CreateCollection {
                name: "Test Collection".to_string(),
                uri: "https://example.com/collection".to_string(),
            }
            .data(),
        };
        let collection = self.collection.insecure_clone();
        self.send(ix, &[&collection])
    }

    fn mint_asset(&mut self) -> TransactionResult {
        let ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new(self.nft.pubkey(), true),
                AccountMeta::new(self.collection.pubkey(), true),
                AccountMeta::new_readonly(self.update_authority, false),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new_readonly(mpl_core_id(), false),
            ],
            data: week5_assgn::instruction::MintAssets {
                name: "Test NFT".to_string(),
                uri: "https://example.com/nft".to_string(),
            }
            .data(),
        };
        let nft = self.nft.insecure_clone();
        let collection = self.collection.insecure_clone();
        self.send(ix, &[&nft, &collection])
    }

    fn initialize(&mut self) -> TransactionResult {
        let ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new(self.config, false),
                AccountMeta::new_readonly(self.collection.pubkey(), false),
                AccountMeta::new_readonly(self.update_authority, false),
                AccountMeta::new(self.rewards_mint, false),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new_readonly(token_program_id(), false),
            ],
            data: week5_assgn::instruction::Initialize {
                rewards_bps: REWARDS_BPS,
                freeze_period: FREEZE_PERIOD_IN_DAYS,
            }
            .data(),
        };
        self.send(ix, &[])
    }

    fn stake(&mut self) -> TransactionResult {
        let ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new_readonly(self.config, false),
                AccountMeta::new(self.nft.pubkey(), false),
                AccountMeta::new(self.collection.pubkey(), false),
                AccountMeta::new_readonly(self.update_authority, false),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new_readonly(mpl_core_id(), false),
            ],
            data: week5_assgn::instruction::Stake {}.data(),
        };
        self.send(ix, &[])
    }

    fn unstake(&mut self) -> TransactionResult {
        let ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new_readonly(self.config, false),
                AccountMeta::new(self.collection.pubkey(), false),
                AccountMeta::new(self.nft.pubkey(), false),
                AccountMeta::new_readonly(self.update_authority, false),
                AccountMeta::new(self.rewards_mint, false),
                AccountMeta::new(self.user_rewards_ata(), false),
                AccountMeta::new_readonly(token_program_id(), false),
                AccountMeta::new_readonly(associated_token_program_id(), false),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new_readonly(mpl_core_id(), false),
            ],
            data: week5_assgn::instruction::Unstake {}.data(),
        };
        self.send(ix, &[])
    }

    fn claim_rewards(&mut self) -> TransactionResult {
        let ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new_readonly(self.config, false),
                AccountMeta::new(self.collection.pubkey(), false),
                AccountMeta::new(self.nft.pubkey(), false),
                AccountMeta::new_readonly(self.update_authority, false),
                AccountMeta::new(self.rewards_mint, false),
                AccountMeta::new(self.user_rewards_ata(), false),
                AccountMeta::new_readonly(token_program_id(), false),
                AccountMeta::new_readonly(associated_token_program_id(), false),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new_readonly(mpl_core_id(), false),
            ],
            data: week5_assgn::instruction::ClaimRewards {}.data(),
        };
        self.send(ix, &[])
    }

    fn burn_staked_nft(&mut self) -> TransactionResult {
        let ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(self.payer.pubkey(), true),
                AccountMeta::new_readonly(self.config, false),
                AccountMeta::new(self.collection.pubkey(), false),
                AccountMeta::new(self.nft.pubkey(), false),
                AccountMeta::new_readonly(self.update_authority, false),
                AccountMeta::new(self.rewards_mint, false),
                AccountMeta::new(self.user_rewards_ata(), false),
                AccountMeta::new_readonly(token_program_id(), false),
                AccountMeta::new_readonly(associated_token_program_id(), false),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new_readonly(mpl_core_id(), false),
            ],
            data: week5_assgn::instruction::BurnStakedNft {}.data(),
        };
        self.send(ix, &[])
    }

    fn transfer_nft(&mut self, new_owner: Address) -> TransactionResult {
        let to_pubkey = |a: Address| anchor_lang::prelude::Pubkey::new_from_array(a.to_bytes());
        let core_ix = mpl_core::instructions::TransferV1Builder::new()
            .asset(to_pubkey(self.nft.pubkey()))
            .collection(Some(to_pubkey(self.collection.pubkey())))
            .payer(to_pubkey(self.payer.pubkey()))
            .new_owner(to_pubkey(new_owner))
            .instruction();
        let ix = Instruction {
            program_id: addr(core_ix.program_id),
            accounts: core_ix
                .accounts
                .iter()
                .map(|m| AccountMeta {
                    pubkey: addr(m.pubkey),
                    is_signer: m.is_signer,
                    is_writable: m.is_writable,
                })
                .collect(),
            data: core_ix.data,
        };
        self.send(ix, &[])
    }

    fn time_travel_days(&mut self, days: i64) {
        let mut clock: Clock = self.svm.get_sysvar();
        clock.unix_timestamp += days * SECONDS_PER_DAY;
        self.svm.set_sysvar(&clock);
    }

    fn token_balance(&self, ata: &Address) -> u64 {
        let account = self.svm.get_account(ata).expect("token account missing");
        u64::from_le_bytes(account.data[64..72].try_into().unwrap())
    }
}

fn assert_custom_error(result: TransactionResult, expected: week5_assgn::error::ErrorCode) {
    let err = result.expect_err("transaction should have failed");
    let expected_code: u32 = expected.into();
    match err.err {
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => {
            assert_eq!(code, expected_code, "logs: {:#?}", err.meta.logs)
        }
        other => panic!("unexpected error {other:?}, logs: {:#?}", err.meta.logs),
    }
}

#[test]
fn test_create_collection() {
    let mut ctx = Ctx::new();
    let res = ctx.create_collection();
    assert!(res.is_ok(), "{:#?}", res.unwrap_err().meta.logs);
    assert!(ctx.svm.get_account(&ctx.collection.pubkey()).is_some());
}

#[test]
fn test_mint_nft() {
    let mut ctx = Ctx::new();
    ctx.create_collection().unwrap();
    let res = ctx.mint_asset();
    assert!(res.is_ok(), "{:#?}", res.unwrap_err().meta.logs);
    assert!(ctx.svm.get_account(&ctx.nft.pubkey()).is_some());
}

#[test]
fn test_stake_nft() {
    let mut ctx = Ctx::new();
    ctx.create_collection().unwrap();
    ctx.mint_asset().unwrap();
    ctx.initialize().unwrap();
    let res = ctx.stake();
    assert!(res.is_ok(), "{:#?}", res.unwrap_err().meta.logs);
}

#[test]
fn test_unstake_before_freeze_period_fails() {
    let mut ctx = Ctx::new();
    ctx.create_collection().unwrap();
    ctx.mint_asset().unwrap();
    ctx.initialize().unwrap();
    ctx.stake().unwrap();

    assert_custom_error(
        ctx.unstake(),
        week5_assgn::error::ErrorCode::FreezePeriodNotElapsed,
    );
}

#[test]
fn test_unstake_after_freeze_period() {
    let mut ctx = Ctx::new();
    ctx.create_collection().unwrap();
    ctx.mint_asset().unwrap();
    ctx.initialize().unwrap();
    ctx.stake().unwrap();

    ctx.time_travel_days(TIME_TRAVEL_IN_DAYS);

    let res = ctx.unstake();
    assert!(res.is_ok(), "{:#?}", res.unwrap_err().meta.logs);

    let expected = TIME_TRAVEL_IN_DAYS as u64
        * REWARDS_BPS as u64
        * 10u64.pow(REWARDS_DECIMALS)
        / 10000;
    assert_eq!(ctx.token_balance(&ctx.user_rewards_ata()), expected);

    let res = ctx.transfer_nft(Address::new_unique());
    assert!(res.is_ok(), "{:#?}", res.unwrap_err().meta.logs);
}

fn expected_rewards(days: i64) -> u64 {
    days as u64 * REWARDS_BPS as u64 * 10u64.pow(REWARDS_DECIMALS) / 10000
}

#[test]
fn test_claim_rewards_keeps_nft_staked_and_frozen() {
    let mut ctx = Ctx::new();
    ctx.create_collection().unwrap();
    ctx.mint_asset().unwrap();
    ctx.initialize().unwrap();
    ctx.stake().unwrap();

    ctx.time_travel_days(TIME_TRAVEL_IN_DAYS);

    let res = ctx.claim_rewards();
    assert!(res.is_ok(), "{:#?}", res.unwrap_err().meta.logs);
    assert_eq!(
        ctx.token_balance(&ctx.user_rewards_ata()),
        expected_rewards(TIME_TRAVEL_IN_DAYS)
    );

    let res = ctx.transfer_nft(Address::new_unique());
    assert!(res.is_err(), "transfer should fail while the NFT is staked");
}

#[test]
fn test_claim_rewards_twice_does_not_double_pay() {
    let mut ctx = Ctx::new();
    ctx.create_collection().unwrap();
    ctx.mint_asset().unwrap();
    ctx.initialize().unwrap();
    ctx.stake().unwrap();

    ctx.time_travel_days(TIME_TRAVEL_IN_DAYS);
    ctx.claim_rewards().unwrap();

    let res = ctx.claim_rewards();
    assert!(res.is_ok(), "{:#?}", res.unwrap_err().meta.logs);
    assert_eq!(
        ctx.token_balance(&ctx.user_rewards_ata()),
        expected_rewards(TIME_TRAVEL_IN_DAYS)
    );

    ctx.time_travel_days(2);
    ctx.claim_rewards().unwrap();
    assert_eq!(
        ctx.token_balance(&ctx.user_rewards_ata()),
        expected_rewards(TIME_TRAVEL_IN_DAYS + 2)
    );
}

const BURN_BONUS_TOKENS: u64 = 1000;

#[test]
fn test_burn_staked_nft() {
    let mut ctx = Ctx::new();
    ctx.create_collection().unwrap();
    ctx.mint_asset().unwrap();
    ctx.initialize().unwrap();
    ctx.stake().unwrap();

    ctx.time_travel_days(TIME_TRAVEL_IN_DAYS);

    let res = ctx.burn_staked_nft();
    assert!(res.is_ok(), "{:#?}", res.unwrap_err().meta.logs);

    assert_eq!(
        ctx.token_balance(&ctx.user_rewards_ata()),
        expected_rewards(TIME_TRAVEL_IN_DAYS) + BURN_BONUS_TOKENS * 10u64.pow(REWARDS_DECIMALS)
    );

    let asset_len = ctx.svm.get_account(&ctx.nft.pubkey()).map_or(0, |a| a.data.len());
    assert!(asset_len <= 1, "asset still has {asset_len} bytes of data");
}

#[test]
fn test_burn_unstaked_nft_fails() {
    let mut ctx = Ctx::new();
    ctx.create_collection().unwrap();
    ctx.mint_asset().unwrap();
    ctx.initialize().unwrap();

    assert_custom_error(
        ctx.burn_staked_nft(),
        week5_assgn::error::ErrorCode::AssetNotStaked,
    );
}

fn total_staked(ctx: &Ctx) -> Option<u64> {
    let data = ctx.svm.get_account(&ctx.collection.pubkey())?.data;
    let key = b"total_staked";
    let start = data.windows(key.len()).position(|w| w == key)? + key.len();
    let len = u32::from_le_bytes(data[start..start + 4].try_into().unwrap()) as usize;
    let value = std::str::from_utf8(&data[start + 4..start + 4 + len]).unwrap();
    Some(value.parse().unwrap())
}

#[test]
fn test_total_staked_increments_on_stake_and_decrements_on_unstake() {
    let mut ctx = Ctx::new();
    ctx.create_collection().unwrap();
    ctx.mint_asset().unwrap();
    ctx.initialize().unwrap();
    assert_eq!(total_staked(&ctx), None);

    ctx.stake().unwrap();
    assert_eq!(total_staked(&ctx), Some(1));

    ctx.time_travel_days(TIME_TRAVEL_IN_DAYS);
    let res = ctx.unstake();
    assert!(res.is_ok(), "{:#?}", res.unwrap_err().meta.logs);
    assert_eq!(total_staked(&ctx), Some(0));
}

#[test]
fn test_total_staked_decrements_on_burn() {
    let mut ctx = Ctx::new();
    ctx.create_collection().unwrap();
    ctx.mint_asset().unwrap();
    ctx.initialize().unwrap();

    ctx.stake().unwrap();
    assert_eq!(total_staked(&ctx), Some(1));

    let res = ctx.burn_staked_nft();
    assert!(res.is_ok(), "{:#?}", res.unwrap_err().meta.logs);
    assert_eq!(total_staked(&ctx), Some(0));
}
