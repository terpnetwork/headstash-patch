use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{ CosmosMsg, Empty};

#[cw_serde]
pub struct Member {
    pub address: String,
    pub mint_count: u32,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub members: Vec<Member>,
    pub member_limit: u32, // 1
    pub admins: Vec<String>,
    pub admins_mutable: bool,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddMembers(AddMembersMsg),
    UpdateAdmins { admins: Vec<String> },
    Freeze {},
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
    Member { member: String },
    #[returns(ConfigResponse)]
    Config {},
    #[returns(AdminListResponse)]
    AdminList {},
    #[returns(CanExecuteResponse)]
    CanExecute {
        sender: String,
        msg: CosmosMsg<Empty>,
    },
    #[returns(PerAddressLimitResponse)]
    PerAddressLimit {},
}

#[cw_serde]
pub struct MembersResponse {
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
    pub per_address_limit: u32,
    pub member_limit: u32,
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
pub struct PerAddressLimitResponse {
    pub limit: u64,
}
