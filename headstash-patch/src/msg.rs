use cosmwasm_schema::{cw_serde, QueryResponses};
use cw4::Member;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub members: Vec<Member>,
    pub cw4_voting_id: u64,
    pub claim_msg_plaintext: String,
}

#[cw_serde]
pub struct AirdropClaimResponse {
    result: bool,
    amount: u32,
}

#[cw_serde]
pub enum ExecuteMsg {
    ClaimAirdrop {
        eth_address: String,
        eth_sig: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(bool)]
    HeadstashEligible { eth_address: String },
    #[returns()]
    HeadstashAmountAtHeight
}
