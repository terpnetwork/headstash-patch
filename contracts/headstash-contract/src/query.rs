use crate::{msg::QueryMsg, state::CONFIG};
use cosmwasm_std::{entry_point, to_binary, Binary};
use cosmwasm_std::{ Env};
use cosmwasm_std::{Deps,  StdResult};
use cw_goop::helpers::interface::CwGoopContract;


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::HeadstashEligible { eth_address } => {
            to_binary(&query_headstash_is_eligible(deps, eth_address)?)
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
    match config.cw_goop_address {
        Some(address) => CwGoopContract(deps.api.addr_validate(&address)?)
            .includes(&deps.querier, eth_address),
        None => Err(cosmwasm_std::StdError::NotFound {
            kind: "Cw Goop Contract".to_string(),
        }),
    }
}


pub fn query_claim_limit(deps: &Deps) -> StdResult<u32> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config.claim_limit)
}
