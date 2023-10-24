use crate::claim_headstash::claim_headstash;
#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::CONFIG;
use cosmwasm_std::entry_point;
use cosmwasm_std::{ DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use build_message::{state_config, cw_goop_instantiate};
use validation::validate_instantiation_params;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:headstash-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    validate_instantiation_params(info.clone(), msg.clone())?;
    
    let mut res = Response::new();
   
    let cfg = state_config(deps.as_ref(), info.clone(), msg.clone())?;
    CONFIG.save(deps.storage, &cfg)?;
    Ok(res
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("sender", info.sender)
        .add_submessage(cw_goop_instantiate(env, msg)?))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ClaimHeadstash {
            eth_address,
            eth_sig,
        } => claim_headstash(deps, info, _env, eth_address, eth_sig),
    }
}

mod build_message {
    use super::*;
    use crate::state::Config;
    use cosmwasm_std::{to_binary, Deps, WasmMsg, SubMsg};
    use cw_goop::msg::InstantiateMsg as HGInstantiateMsg;

    pub const HEADSTASH_GOOP_LABEL: &str = "Headstash Goop Recipients";
    pub const INIT_CW_GOOP_ID: u64 = 1;

    pub fn cw_goop_instantiate(
        env: Env,
        msg: InstantiateMsg,
    ) -> Result<cosmwasm_std::SubMsg<>, ContractError> {
        let cw_goop_instantiate_msg = HGInstantiateMsg {
           members: msg.members,
           claim_limit: msg.claim_limit,
           admins: msg.admin,
           admins_mutable: msg.admins_mutable,
        };
        let wasm_msg = WasmMsg::Instantiate {
            code_id: msg.cw_goop_id,
            admin: Some(env.contract.address.to_string()),
            funds: vec![],
            label: HEADSTASH_GOOP_LABEL.to_string(),
            msg: to_binary(&cw_goop_instantiate_msg)?,
        };
        Ok(SubMsg::reply_on_success(wasm_msg, INIT_CW_GOOP_ID))
    }

    pub fn state_config(
        deps: Deps,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Config, ContractError> {
        Ok(Config {
            admin: info.sender,
            claim_msg_plaintext: msg.clone().claim_msg_plaintext,
            cw_goop_address: None,
        })
    }

}


mod validation {
    use super::*;

    const MIN_AIRDROP: u128 = 1_061_678_463; // 1,061.678463 TERP
    const MAX_AIRDROP: u128 = 34_093_755_162; // 34,093.755162 TERP

    pub fn validate_instantiation_params(
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<(), ContractError> {
        // validate_headstash_amount(msg.headstash_amount)?;
        validate_plaintext_msg(msg.claim_msg_plaintext)?;
        // validate_instantiate_funds(info)?;
        Ok(())
    }

    pub fn validate_plaintext_msg(plaintext_msg: String) -> Result<(), ContractError> {
        if !plaintext_msg.contains("{wallet}") {
            return Err(ContractError::PlaintextMsgNoWallet {});
        }
        if plaintext_msg.len() > 1000 {
            return Err(ContractError::PlaintextTooLong {});
        }
        Ok(())
    }
}