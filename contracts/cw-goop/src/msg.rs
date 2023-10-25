use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{ CosmosMsg, Empty, Addr};
use crate::state::Config;

#[cw_serde]
pub struct Member {
    pub address: String,
    pub headstash_amount: u128,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub members: Vec<Member>,
    pub admin: Addr,
    pub admin_mutable: bool,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddMembers(AddMembersMsg),
    Freeze {},
}

#[cw_serde]
pub struct AdminResponse {
    pub admin: Addr,
    pub mutable: bool,
}

#[cw_serde]
pub struct AddMembersMsg {
    pub to_add: Vec<Member>,
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
    Member { member: String },
    #[returns(ConfigResponse)]
    Config {},
    #[returns(AdminResponse)]
    AdminList {},
    #[returns(CanExecuteResponse)]
    CanExecute {
        sender: String,
        msg: CosmosMsg<Empty>,
    },
    #[returns(HeadstashAmountResponse)]
    GetHeadstashAmount {
        address: String,
    },
}

#[cw_serde]
pub struct HasMemberResponse {
    pub has_member: bool,
}

#[cw_serde]
pub struct HeadstashAmountResponse{
    pub headstash_amount: u128,
}

#[cw_serde]
pub struct MemberResponse {
    pub member: Member,
}

#[cw_serde]
pub struct MembersResponse {
    pub members: Vec<Member>,
}

#[cw_serde]
pub struct ConfigResponse {
    pub num_members: u32,
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
