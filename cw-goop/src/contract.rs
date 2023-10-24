use crate::admin::{
    can_execute, execute_freeze, execute_update_admins, query_admin_list, query_can_execute,
};
use crate::error::ContractError;
use crate::helpers::validators::map_validate;
use crate::msg::{
    AddMembersMsg, ConfigResponse, ExecuteMsg,  HasMemberResponse,HeadstashAmountResponse,
     InstantiateMsg, Member, MembersResponse, QueryMsg,
    
};
use crate::state::{AdminList, Config, ADMIN_LIST, CONFIG, GOOPLIST,HEADSTASH_AMOUNT};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{ to_binary, Binary, Deps, DepsMut, Env, MessageInfo, StdResult, Response};
use cosmwasm_std::{Order};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::{ maybe_addr};


// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-goop";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// headstash constants
pub const MAX_MEMBERS: u32 = 12376; // # of unique addr in headstash allocation
pub const MAX_CLAIM: u32 = 1; // max # of times an address can claim a headstash allocation

// queries
const PAGINATION_DEFAULT_LIMIT: u32 = 25;
const PAGINATION_MAX_LIMIT: u32 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // ensures claim_limit value is valid
    if msg.claim_limit == 0 || msg.claim_limit > MAX_CLAIM {
        return Err(ContractError::InvalidMemberLimit {
            min: 1,
            max: MAX_CLAIM,
            got: msg.claim_limit,
        });
    }

    // sets up the new contracts config.
    let config = Config {
        num_members: msg.members.len() as u32,
        claim_limit: msg.claim_limit,
    };
    CONFIG.save(deps.storage, &config)?;

    // sets up the new contracts admins
    let admin_config = AdminList {
        admins: map_validate(deps.api, &msg.admins)?,
        mutable: msg.admins_mutable,
    };
    ADMIN_LIST.save(deps.storage, &admin_config)?;

    // responds a new smart contract instance
    let res = Response::new();
    
    // if member limit will be less than the number of members, error.
    if MAX_MEMBERS < config.num_members {
        return Err(ContractError::MembersExceeded {
            expected: MAX_MEMBERS,
            actual: config.num_members,
        });
    }

    Ok(res
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("sender", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddMembers(msg) => execute_add_members(deps, env, info, msg),
        ExecuteMsg::UpdateAdmins { admins } => execute_update_admins(deps, env, info, admins),
        ExecuteMsg::Freeze {} => execute_freeze(deps, env, info),
    }
}




pub fn execute_add_members(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: AddMembersMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    // ensure sender can call this function.
    can_execute(&deps, info.sender.clone())?;

    // iterates of adding each new member will exceed max member config
    for add in msg.to_add.into_iter() {
        if config.num_members >= MAX_MEMBERS {
            return Err(ContractError::MembersExceeded {
                expected: MAX_MEMBERS,
                actual: config.num_members,
            });
        }
        let addr = &add.address;
        // checks for duplicates
        if GOOPLIST.has(deps.storage, addr.clone()) {
            return Err(ContractError::DuplicateMember(addr.to_string()));
        }
        // saves new member address to storage
        GOOPLIST.save(deps.storage, addr.to_string(), &add.claim_count)?;
        // increments total number of members
        config.num_members += 1;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "add_members")
        .add_attribute("sender", info.sender))
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Members { start_after, limit } => {
            to_binary(&query_members(deps, start_after, limit)?)
        }
        QueryMsg::HasMember { member } => to_binary(&query_has_member(deps, member)?),
        QueryMsg::Member { member, .. } => to_binary(&query_member(deps, member)?),
        QueryMsg::Config {} => to_binary(&query_config(deps, env)?),
        QueryMsg::AdminList {} => to_binary(&query_admin_list(deps)?),
        QueryMsg::ClaimLimit {} => to_binary(&query_claim_limit(deps)?),
        QueryMsg::CanExecute { sender, .. } => to_binary(&query_can_execute(deps, &sender)?),
        QueryMsg::GetHeadstashAmount {address} => to_binary(&query_get_headstash_amount(deps, address)?)
    }
}



pub fn query_claim_limit(deps: Deps) -> StdResult<u32> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config.claim_limit)
}


pub fn query_members(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<MembersResponse> {
    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;
    let start_addr = maybe_addr(deps.api, start_after)?;
    let start = start_addr.map(Bound::exclusive);
    
    let members: Vec<Member> = GOOPLIST
    .range(deps.storage, start, None, Order::Ascending)
    .take(limit)
    .map(|res| {
        res.map(|(addr, headstash_amount)| {
        let claim_count = GOOPLIST.load(deps.storage, addr.clone()).unwrap_or_default();
            Member {
                address: addr,
                headstash_amount,
                claim_count,
            }
        })
    })
    .collect::<Result<Vec<Member>, _>>()?;


    Ok(MembersResponse { members })
}

pub fn query_get_headstash_amount(deps: Deps, address: String) -> StdResult<HeadstashAmountResponse> {
    let member = query_member(deps, address)?;

    Ok(HeadstashAmountResponse {
        headstash_amount: member.headstash_amount,
    })
}

pub fn query_has_member(deps: Deps, member: String) -> StdResult<HasMemberResponse> {
    let addr = deps.api.addr_validate(&member)?;

    Ok(HasMemberResponse {
        has_member: GOOPLIST.has(deps.storage, addr.to_string()),
    })
}



pub fn query_member(deps: Deps, member: String) -> StdResult<Member> {
    let addr = member;
    let headstash_amount = HEADSTASH_AMOUNT.load(deps.storage, addr.clone())?;
    let claim_count = GOOPLIST.load(deps.storage, addr.clone())?;
    Ok(Member {
        address: addr,
        headstash_amount,
        claim_count,
    })
}

pub fn query_config(deps: Deps, _env: Env) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        num_members: config.num_members,
        claim_limit: config.claim_limit,
        config,
    })
}

