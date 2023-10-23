use crate::{msg::QueryMsg, state::CONFIG, ContractError};
use cosmwasm_std::{entry_point, to_binary, Binary};
use cosmwasm_std::{Addr, Env};
use cosmwasm_std::{Deps, DepsMut, StdResult};
// use vending_minter::helpers::MinterContract;
use cw4::Cw4Contract;
use cw4::{MemberListResponse, MemberResponse, TotalWeightResponse};
use cw4_group::msg::QueryMsg as HeadstashGroupQueryMsg;


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::HeadstashEligible { eth_address } => {
            to_binary(&query_headstash_is_eligible(deps, eth_address)?)
        }
        QueryMsg::HeadstashAmountAtHeight { eth_address, height } => {
            query_headstash_amount_at_height(deps, env, eth_address, height)
        }
        // QueryMsg::GetMinter {} => to_binary(&query_minter(deps)?),
    }
}

// fn query_minter(deps: Deps) -> StdResult<Addr> {
//     let config = CONFIG.load(deps.storage)?;
//     Ok(config.minter_address)
// }

pub fn query_headstash_is_eligible(deps: Deps, eth_address: String) -> StdResult<bool> {
    let config = CONFIG.load(deps.storage)?;
    match config.headstash_group {
        Some(address) => Cw4Contract(deps.api.addr_validate(&address)?)
            .includes(&deps.querier, eth_address),
        None => Err(cosmwasm_std::StdError::NotFound {
            kind: "Headstash Contract".to_string(),
        }),
    }
}

pub fn query_headstash_group(deps: &DepsMut) -> Result<String, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let headstash_group = config.headstash_group;
    let config = HeadstashContract(headstash_group).config(&deps.querier)?;
    match config.headstash_group {
        Some(headstash_group) => Ok(headstash_group),
        None => Err(ContractError::HeadstashGroupNotSet {}),
    }
}

pub fn query_per_address_limit(deps: &Deps) -> StdResult<u32> {
    let config = CONFIG.load(deps.storage)?;
    let max_claim_count = MAX_CLAIM_COUNT.load(deps.storage)?;
    match max_claim_count {
        Some(max_claim_count) => Ok(max_claim_count),
        None => Err(ContractError::MaxClaimCountNotSet {}),
    }
}

pub fn query_voting_power_at_height(
    deps: Deps,
    env: Env,
    eth_address: String,
    height: Option<u64>,
) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    let headstash_group = config.headstash_group;
    let addr = deps.api.addr_validate(&address)?.to_string();
    let res: MemberResponse = deps.querier.query_wasm_smart(
        headstash_group,
        &cw4_group::msg::QueryMsg::Member {
            addr,
            at_height: height,
        },
    )?;

    to_binary(&dao_interface::voting::VotingPowerAtHeightResponse {
        power: res.weight.unwrap_or(0).into(),
        height: height.unwrap_or(env.block.height),
    })
}