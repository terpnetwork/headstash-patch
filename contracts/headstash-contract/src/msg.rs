use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_goop::msg::Member;


#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Vec<String>,
    pub claim_msg_plaintext: String,
    pub members: Vec<Member>,
    pub cw_goop_id: u64,
    pub claim_limit: u32,
    pub admins_mutable: bool,
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
    // #[returns(Addr)]
    // GetMinter {},
}
