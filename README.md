# Headstash Patch Contracts

## [cw-goop](./cw-goop/)
- fork of Stargaze's [whitelist-flex](https://github.com/public-awesome/launchpad/tree/main/contracts/whitelists/whitelist-flex)
- removed `addr_check()` functions preventing the `addr` . 
- Import `Member` into headstash contract, so that we can handle the allocation amounts with the addr `weight`.
## [headstash-contract](./headstash-contract/)
- fork of Stargaze's [sg-eth-airdrop](https://github.com/public-awesome/launchpad/blob/main/contracts/sg-eth-airdrop)