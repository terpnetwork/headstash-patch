use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_goop::msg::Member;
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
    pub claim_msg_plaintext: String,
    pub members: Vec<Member>,
    pub cw_goop_id: u64,
    pub per_address_limit: u32,
}

#[cw_serde]
pub struct HeadstashClaimResponse {
    result: bool,
    amount: u32,
}

#[cw_serde]
pub enum ExecuteMsg {
    ClaimHeadstash {
        eth_address: String,
        eth_sig: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(bool)]
    HeadstashEligible { eth_address: String },
}
