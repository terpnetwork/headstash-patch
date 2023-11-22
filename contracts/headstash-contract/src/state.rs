use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    /// Owner If None set, contract is frozen.
    pub owner: Addr,
    pub claim_msg_plaintext: String,
}
pub const CONFIG: Item<Config> = Item::new("config");

pub const NATIVE_FEE_DENOM: &str = "uterpx";
pub const NATIVE_BOND_DENOM: &str = "uthiolx";

// saves external network airdrop accounts
pub const ACCOUNT_MAP_KEY: &str = "account_map";
// external_address -> host_address
pub const ACCOUNT_MAP: Map<String, String> = Map::new(ACCOUNT_MAP_KEY);

pub const MERKLE_ROOT_PREFIX: &str = "merkle_root";
pub const MERKLE_ROOT: Item<String> = Item::new(MERKLE_ROOT_PREFIX);

pub const AMOUNT_KEY: &str = "amount";
pub const AMOUNT: Item<Uint128> = Item::new(AMOUNT_KEY);

pub const CLAIM_PREFIX: &str = "claim";
pub const CLAIM: Map<String, bool> = Map::new(CLAIM_PREFIX);

pub const AMOUNT_CLAIMED_KEY: &str = "claimed_amount";
pub const AMOUNT_CLAIMED: Item<Uint128> = Item::new(AMOUNT_CLAIMED_KEY);

pub const PAUSED_KEY: &str = "paused";
pub const PAUSED: Item<bool> = Item::new(PAUSED_KEY);