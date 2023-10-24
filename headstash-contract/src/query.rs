use crate::{msg::QueryMsg, state::CONFIG, ContractError};
use cosmwasm_std::{ Env, Deps,  StdResult, entry_point, to_binary, Binary};
use cw_goop::helpers::interface::CwGoopContract;
// use cw_goop::msg::Member;



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

pub fn query_headstash_goop(deps: Deps, _env: Env) -> Result<String, ContractError> {
    // Load the Config from storage
    let config = CONFIG.load(deps.storage)?;

    // Check if cw_goop_address is Some and unwrap it, or provide a default if it's None
    let cw_goop_address = config.cw_goop_address.unwrap_or("default_value".to_string());

    // Now, cw_goop_address contains the value of the cw_goop_address field
    Ok(cw_goop_address)
}


pub fn query_claim_limit(deps: &Deps) -> StdResult<u32> {
    let config = CONFIG.load(deps.storage)?;

    match config.cw_goop_address {
        Some(address) => CwGoopContract(deps.api.addr_validate(&address)?)
            .claim_limit(&deps.querier),
        None => Err(cosmwasm_std::StdError::NotFound {
            kind: "Whitelist Contract".to_string(),
        }),
    }
}
