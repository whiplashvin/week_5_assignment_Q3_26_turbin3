# week_5_assignment_Q3_26_turbin3

NFT Staking with Metaplex Core

An Anchor program for staking Metaplex Core NFTs. Staked NFTs are frozen in the owner's wallet and earn SPL token rewards over time. The program never takes custody of the NFT.

## Features

- Create a Core collection and mint NFTs into it, with a program PDA as the update authority
- Stake an NFT by freezing it in place (`FreezeDelegate` plugin) and recording the stake on the NFT itself (`Attributes` plugin)
- Unstake after a configurable freeze period and receive rewards for every full day staked
- Claim rewards without unstaking; the NFT stays staked and frozen
- Burn a staked NFT (`BurnDelegate` plugin) for its staking rewards plus a one-time bonus
- Track `total_staked` for the whole collection as an `Attributes` plugin on the collection account

### Build

```sh
anchor build
```

### Test

```sh
anchor build
anchor test   # or: cargo test
```

The tests also load the MPL Core program from `tests/fixtures/mpl_core.so`. To refresh it from mainnet:

```sh
solana program dump -um CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d programs/week5_assgn/tests/fixtures/mpl_core.so
```
