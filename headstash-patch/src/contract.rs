use crate::claim_headstash::claim_headstash;
#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::CONFIG;
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw4::{MemberListResponse, MemberResponse, TotalWeightResponse};

use build_message::{state_config, headstash_instantiate};
use validation::validate_instantiation_params;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:headstash-airdrop";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    validate_instantiation_params(info.clone(), msg.clone())?;
    let mut res = Response::new();
    // fair_burn(INSTANTIATION_FEE, None, &mut res);
    let cfg = state_config(deps.as_ref(), info.clone(), msg.clone())?;
    CONFIG.save(deps.storage, &cfg)?;
    Ok(res
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("sender", info.sender)
        .add_submessage(headstash_instantiate(env, msg)?))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ClaimAirdrop {
            eth_address,
            eth_sig,
        } => claim_headstash(deps, info, _env, eth_address, eth_sig),
    }
}

mod build_message {
    use super::*;
    use crate::state::Config;
    use cosmwasm_std::{to_binary, Deps, WasmMsg, SubMsg, CosmosMsg};
    use validation::validate_headstash_amount;
    use cw4_group::msg::InstantiateMsg as HGInstantiateMsg;

    pub const HEADSTASH_GROUP_LABEL: &str = "Headstash Group for Airdrop";
    pub const INIT_HEADSTASH_GROUP_ID: u64 = 1;

    pub fn headstash_instantiate(
        env: Env,
        msg: InstantiateMsg,
    ) -> Result<cosmwasm_std::SubMsg<CosmosMsg>, ContractError> {
        let headstash_group_instantiate_msg = HGInstantiateMsg {
            admin: msg.admin,
            members: msg.members,
            cw4_group_id: msg.cw4_group_id
            // mint_discount_bps: Some(0),
            // per_address_limit: msg.per_address_limit,
        };
        let wasm_msg = WasmMsg::Instantiate {
            code_id: msg.cw4_group_id,
            admin: Some(env.contract.address.to_string()),
            funds: vec![],
            label: HEADSTASH_GROUP_LABEL.to_string(),
            msg: to_binary(&headstash_group_instantiate_msg)?,
        };
        Ok(SubMsg::reply_on_success(wasm_msg, INIT_HEADSTASH_GROUP_ID))
    }

    pub fn state_config(
        deps: Deps,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Config, ContractError> {
        Ok(Config {
            admin: info.sender,
            claim_msg_plaintext: msg.clone().claim_msg_plaintext,
            headstash_group_address: None,
            // minter_address: deps.api.addr_validate(msg.minter_address.as_ref())?,
        })
    }

}


mod validation {
    use super::*;
    use cosmwasm_std::Uint128;
    use cw_utils::must_pay;
    use crate::state::NATIVE_DENOM;

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

    // pub fn validate_instantiate_funds(info: MessageInfo) -> Result<(), ContractError> {
    //     let amount = must_pay(&info, NATIVE_DENOM)?;
    //     if amount < Uint128::from(INSTANTIATION_FEE) {
    //         return Err(ContractError::InsufficientFundsInstantiate {});
    //     };
    //     Ok(())
    // }

    // pub fn validate_headstash_amount(headstash_amount: u128) -> Result<u128, ContractError> {
    //     if headstash_amount < MIN_AIRDROP {
    //         return Err(ContractError::AirdropTooSmall {});
    //     };
    //     if headstash_amount > MAX_AIRDROP {
    //         return Err(ContractError::AirdropTooBig {});
    //     };
    //     Ok(headstash_amount)
    // }

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