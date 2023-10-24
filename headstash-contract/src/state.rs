use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Config {
    pub admin: Addr,
    pub claim_msg_plaintext: String,
    pub cw_goop_address: Option<String>,
}


pub const NATIVE_BOND_DENOM: &str = "uterp";
pub const NATIVE_FEE_DENOM: &str = "uthiol";
pub const MAX_CLAIM_COUNT: u32 = 1;


pub const CONFIG: Item<Config> = Item::new("cfg");

pub const ADDRS_CLAIM_COUNT: Map<&str, u32> = Map::new("acc");