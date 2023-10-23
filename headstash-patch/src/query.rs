use crate::{msg::QueryMsg, state::CONFIG, ContractError};
use cosmwasm_std::{ Env, Deps, DepsMut, StdResult, entry_point, to_binary, Binary};
use cw_goop::helpers::CwGoopContract;
use cw_goop::msg::Member;



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::HeadstashEligible { eth_address } => {
            to_binary(&query_headstash_is_eligible(deps, eth_address)?)
        }
    }
}

// fn query_minter(deps: Deps) -> StdResult<Addr> {
//     let config = CONFIG.load(deps.storage)?;
//     Ok(config.minter_address)
// }

pub fn query_headstash_is_eligible(deps: Deps, eth_address: String) -> StdResult<bool> {
    let config = CONFIG.load(deps.storage)?;
    match config.cw_goop_address {
        Some(address) => CwGoopContract(deps.api.addr_validate(&address)?)
            .includes(&deps.querier, eth_address),
        None => Err(cosmwasm_std::StdError::NotFound {
            kind: "Cw-Goop Contract".to_string(),
        }),
    }
}

pub fn query_headstash_goop(deps: &DepsMut) -> Result<String, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let cw_goop = config.cw_goop_address;
    let config = CwGoopContract(cw_goop).config(&deps.querier)?;
    match config.cw_goop_address {
        Some(cw_goop) => Ok(cw_goop),
        None => Err(ContractError::HeadstashGroupNotSet {}),
    }
}

pub fn query_per_address_limit(deps: &Deps) -> StdResult<u32> {
    let config = CONFIG.load(deps.storage)?;
    match config.cw_goop_address {
        Some(address) => CwGoopContract(address)
            .per_address_limit(&deps.querier),
        None => Err(cosmwasm_std::StdError::NotFound {
            kind: "Whitelist Contract".to_string(),
        }),
    }
}
