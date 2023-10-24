use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{ CosmosMsg, Empty};
use crate::state::Config;


#[cw_serde]
pub struct Member {
    pub address: String, // Ox24EaSp0...
    pub headstash_amount: u32,
    pub claim_count: u32, // # of claims. Start @ 0, never more than 1.

}

#[cw_serde]
pub struct InstantiateMsg {
    pub members: Vec<Member>,
    pub claim_limit: u32, // 1
    pub admins: Vec<String>,
    pub admins_mutable: bool,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddMembers(AddMembersMsg), // add members to contract
    UpdateAdmins { admins: Vec<String> }, // update the admins of contract
    Freeze {}, // freeze contract state
}

#[cw_serde]
pub struct AdminListResponse { 
    pub admins: Vec<String>,
    pub mutable: bool,
}

#[cw_serde]
pub struct AddMembersMsg { 
    pub to_add: Vec<Member>,
}

#[cw_serde]
pub struct RemoveMembersMsg {
    pub to_remove: Vec<String>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(MembersResponse)]
    Members {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(HasMemberResponse)]
    HasMember { member: String },
    #[returns(MemberResponse)]
    Member { member: String, claim_count: u32 },
    #[returns(ConfigResponse)]
    Config {},
    #[returns(AdminListResponse)]
    AdminList {},
    #[returns(CanExecuteResponse)]
    CanExecute {
        sender: String,
        msg: CosmosMsg<Empty>,
    },
    #[returns(ClaimLimitResponse)]
    ClaimLimit {},
}

#[cw_serde]
pub struct MembersResponse { //returns a the vector of members.
    pub members: Vec<Member>,
}

#[cw_serde]
pub struct HasMemberResponse {
    pub has_member: bool,
}

#[cw_serde]
pub struct MemberResponse {
    pub member: Member,
}



#[cw_serde]
pub struct ConfigResponse {
    pub num_members: u32,
    pub claim_limit: u32,
    pub config: Config,
}

#[cw_serde]
pub enum SudoMsg {
    /// Add a new operator
    AddOperator { operator: String },
    /// Remove operator
    RemoveOperator { operator: String },
}

#[cw_serde]
pub struct CanExecuteResponse {
    pub can_execute: bool,
}

#[cw_serde]
pub struct ClaimLimitResponse {
    pub limit: u32,
}
