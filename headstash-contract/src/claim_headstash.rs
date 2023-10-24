use crate::query::query_headstash_goop;
use crate::state::ADDRS_CLAIM_COUNT;
use crate::{state::CONFIG, ContractError};
use build_messages::claim_and_headstash_add;
use cosmwasm_std::{coins, DepsMut,  BankMsg, StdResult, Env, MessageInfo, Response, CosmosMsg, SubMsg};
use cw_goop::msg::ExecuteMsg as CwGoopContractExecuteMsg;
use cw_goop::{helpers::interface::CwGoopContract, msg::AddMembersMsg};
use cw_goop::msg::Member;
use validation::validate_claim;


pub const DEFAULT_CLAIM_COUNT: u32 = 0; 


pub fn claim_headstash(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    eth_address: String,
    eth_sig: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    validate_claim(
        &deps,
        info.clone(),
        eth_address.clone(),
        eth_sig,
        config.clone(),
    )?;

    // Load the Member from storage using the eth_address
    let member: Member = load_goop_member(deps.storage, &eth_address)?;

    let headstash_amount = member.mint_count;


    let res = claim_and_headstash_add(&deps, info, headstash_amount)?;
    increment_local_claim_count_for_address(deps, eth_address)?;

    Ok(res.add_attribute("claimed_amount", headstash_amount.to_string()))
}



pub fn increment_local_claim_count_for_address(
    deps: DepsMut,
    eth_address: String,
) -> Result<Response, ContractError> {
    let claim_count_for_address = ADDRS_CLAIM_COUNT
        .load(deps.storage, &eth_address)
        .unwrap_or(0);
    ADDRS_CLAIM_COUNT.save(deps.storage, &eth_address, &(claim_count_for_address + 1))?;

    Ok(Response::new())
}

mod build_messages {
    use super::*;
    use crate::{state::NATIVE_BOND_DENOM};

    pub fn claim_and_headstash_add(
        deps: &DepsMut,
        info: MessageInfo,
        headstash_amount: u128,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        let mut res = Response::new();
       
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: coins(headstash_amount, NATIVE_BOND_DENOM),
        });
        res = res.add_submessage(bank_msg);
        let headstash_goop_address = query_headstash_goop(deps.as_ref(), env)?;
        let res = res.add_message(add_member_to_headstash_goop_address(
            deps,
            info.sender.to_string(),
            1,
            headstash_goop_address,
        )?);

        Ok(res)
    }

    fn add_member_to_headstash_goop_address(
        deps: &DepsMut,
        member_address: String,
        headstash_amount: u32, 
        headstash_goop_address: String,
    ) -> StdResult<CosmosMsg> {
        let inner_msg = AddMembersMsg {
                to_add: vec![Member {
                    address: member_address.to_string(),
                    headstash_amount,
                    claim_count: DEFAULT_CLAIM_COUNT,
                }],
        };
        let execute_msg = CwGoopContractExecuteMsg::AddMembers(inner_msg);
            CwGoopContract(deps.api.addr_validate(&headstash_goop_address)?)
            .call(execute_msg)
    }
}

mod validation {
    use super::*;
    use cosmwasm_std::StdError;
    use ethereum_verify::verify_ethereum_text;

    use crate::{
        query::{query_headstash_is_eligible, query_claim_limit},
        state::Config,
    };

    pub fn compute_plaintext_msg(config: &Config, info: MessageInfo) -> String {
        str::replace(
            &config.claim_msg_plaintext,
            "{wallet}",
            info.sender.as_ref(),
        )
    }

    pub fn validate_claim(
        deps: &DepsMut,
        info: MessageInfo,
        eth_address: String,
        eth_sig: String,
        config: Config,
    ) -> Result<(), ContractError> {
        validate_is_eligible(deps, eth_address.clone())?;
        validate_eth_sig(deps, info, eth_address.clone(), eth_sig, config)?;
        validate_claims_remaining(deps, &eth_address)?;
        Ok(())
    }

    fn validate_is_eligible(deps: &DepsMut, eth_address: String) -> Result<(), ContractError> {
        let eligible = query_headstash_is_eligible(deps.as_ref(), eth_address.clone())?;
        match eligible {
            true => Ok(()),
            false => Err(ContractError::AddressNotEligible {
                address: eth_address,
            }),
        }
    }

    fn validate_eth_sig(
        deps: &DepsMut,
        info: MessageInfo,
        eth_address: String,
        eth_sig: String,
        config: Config,
    ) -> Result<(), ContractError> {
        let valid_eth_sig =
            validate_ethereum_text(deps, info, &config, eth_sig, eth_address.clone())?;
        match valid_eth_sig {
            true => Ok(()),
            false => Err(ContractError::AddressNotEligible {
                address: eth_address,
            }),
        }
    }

    pub fn validate_claims_remaining(
        deps: &DepsMut,
        eth_address: &str,
    ) -> Result<(), ContractError> {
        let claim_count = ADDRS_CLAIM_COUNT.load(deps.storage, eth_address);
        let claim_count = claim_count.unwrap_or(0);
        let claim_limit = query_claim_limit(&deps.as_ref())?;
        if claim_count < claim_limit {
            Ok(())
        } else {
            Err(ContractError::ClaimCountReached {
                address: eth_address.to_string(),
            })
        }
    }

    pub fn validate_ethereum_text(
        deps: &DepsMut,
        info: MessageInfo,
        config: &Config,
        eth_sig: String,
        eth_address: String,
    ) -> StdResult<bool> {
        let plaintext_msg = compute_plaintext_msg(config, info);
        match hex::decode(eth_sig.clone()) {
            Ok(eth_sig_hex) => {
                verify_ethereum_text(deps.as_ref(), &plaintext_msg, &eth_sig_hex, &eth_address)
            }
            Err(_) => Err(StdError::InvalidHex {
                msg: format!("Could not decode {eth_sig}"),
            }),
        }
    }
}