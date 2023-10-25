
#[cfg(not(feature = "library"))]
use crate::claim_headstash::claim_headstash;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::CONFIG;

use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use build_message::{state_config, cw_goop_instantiate};
use validation::validate_instantiation_params;

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
    let res = Response::new();
    
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
    ) -> Result<cosmwasm_std::SubMsg<>, ContractError> { // prev: SubMsg<StargazeMsgWrapper>
        let cw_goop_instantiate_msg = HGInstantiateMsg {
            members: msg.members,
            admin: msg.admin,
            admin_mutable: msg.admin_mutable,
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
        _deps: Deps,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Config, ContractError> {
        Ok(Config {
            admin: info.sender,
            claim_msg_plaintext: msg.clone().claim_msg_plaintext,
            cw_goop_address: None,
            claim_limit: msg.claim_limit,
        })
    }
}

mod validation {
    use super::*;

    pub fn validate_instantiation_params(
        _info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<(), ContractError> {
        validate_plaintext_msg(msg.claim_msg_plaintext)?;
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