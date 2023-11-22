#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, coin,to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128, BankMsg, CosmosMsg, DistributionMsg,
};
use cw2::{get_contract_version, set_contract_version};
use crate::contract::validation::validate_claim;
use sha2::Digest;
use std::convert::TryInto;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, IsClaimedResponse,
    MerkleRootResponse, MigrateMsg, QueryMsg, TotalClaimedResponse,
};
use crate::state::{
    Config, NATIVE_FEE_DENOM, NATIVE_BOND_DENOM, CLAIM, CONFIG, MERKLE_ROOT, AMOUNT_CLAIMED,
     PAUSED, AMOUNT
};

// Version info, for migration info
const CONTRACT_NAME: &str = "crates.io:cw20-vesting-airdrop";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // set paused state to false
    PAUSED.save(deps.storage, &false)?;

    // define config
    let config = Config {
        owner: info.sender,
        claim_msg_plaintext: msg.claim_msg_plaintext,
    };
    CONFIG.save(deps.storage, &config)?;

    // check merkle root length
    let mut root_buf: [u8; 32] = [0; 32];
    hex::decode_to_slice(&msg.merkle_root, &mut root_buf)?;

    MERKLE_ROOT.save(deps.storage,  &msg.merkle_root)?;

    // save total airdropped amount
    let amount = msg.total_amount.unwrap_or_else(Uint128::zero);
    AMOUNT.save(deps.storage, &amount)?;
    AMOUNT_CLAIMED.save(deps.storage, &Uint128::zero())?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "instantiate"),
        attr("merkle_root", msg.merkle_root),
        attr("total_amount", amount),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Claim { amount, proof, eth_pubkey, eth_sig } => execute_claim(deps, env, info, eth_pubkey, eth_sig, amount, proof ),
        ExecuteMsg::ClawBack { recipient } => {execute_clawback(deps, env, info, Some(recipient))},
        ExecuteMsg::Pause {} => execute_pause(deps, env, info),
        ExecuteMsg::Resume {} => execute_resume(deps,env,info)
    }
}

pub fn execute_claim(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    eth_pubkey: String,
    eth_sig: String,
    amount: Uint128,
    proof: Vec<String>,
) -> Result<Response, ContractError> {

    let is_paused = PAUSED.load(deps.storage)?;
    if is_paused {
        return Err(ContractError::Paused {});
    }

    // verify not claimed
    let claimed = CLAIM.may_load(deps.storage, eth_pubkey.clone())?;
    if claimed.is_some() {
        return Err(ContractError::Claimed {});
    }

    // verify merkle root
    let config = CONFIG.load(deps.storage)?;
    let merkle_root = MERKLE_ROOT.load(deps.storage)?;

    // validate the eth_sig was generated with the eth_pubkey provided
    validate_claim( &deps,
        info.clone(),
        eth_pubkey.clone(),
        eth_sig,
        config.clone(),
    )?;

    // generate merkleTree leaf with eth_pubkey & amount
    let user_input = format!("{}{}", eth_pubkey, amount);
    let hash = sha2::Sha256::digest(user_input.as_bytes())
        .as_slice()
        .try_into()
        .map_err(|_| ContractError::WrongLength {})?;

    let hash = proof.into_iter().try_fold(hash, |hash, p| {
        let mut proof_buf = [0; 32];
        hex::decode_to_slice(p, &mut proof_buf)?;
        let mut hashes = [hash, proof_buf];
        hashes.sort_unstable();
        sha2::Sha256::digest(&hashes.concat())
            .as_slice()
            .try_into()
            .map_err(|_| ContractError::WrongLength {})
    })?;

    let mut root_buf: [u8; 32] = [0; 32];
    hex::decode_to_slice(merkle_root, &mut root_buf)?;
    if root_buf != hash {
        return Err(ContractError::VerificationFailed {});
    }

    // update claim index
    CLAIM.save(deps.storage, eth_pubkey, &true)?;

    // Update total claimed to reflect
    let mut claimed_amount = AMOUNT_CLAIMED.load(deps.storage)?;
    claimed_amount += amount;
    AMOUNT_CLAIMED.save(deps.storage, &claimed_amount)?;
    
    let bank_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![coin(amount.u128(), NATIVE_BOND_DENOM),
                     coin(amount.u128(), NATIVE_FEE_DENOM),],
    });

    let res = Response::new()
        .add_message(bank_msg)
        .add_attributes(vec![
            attr("action", "claim"),
            attr("address", info.sender),
            attr("amount", amount),
        ]);
    Ok(res)
}

pub fn execute_clawback(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _recipient: Option<String>,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    
    // authorize owner
    if info.sender != cfg.owner {
        return Err(ContractError::Unauthorized {})
    }

    // error if contract is paused
    let is_paused = PAUSED.load(deps.storage)?;
    if is_paused {
        return Err(ContractError::Paused {});
    }

    let claimed = AMOUNT_CLAIMED.load(deps.storage)?;
    let total_amount = AMOUNT.load(deps.storage)?;
    // get balance
    let balance_to_burn = total_amount.checked_sub(claimed)?;

    // clawback to community pool
    let clawback_msg = CosmosMsg::Distribution(DistributionMsg::FundCommunityPool {
        amount: vec![coin(balance_to_burn.u128(), NATIVE_BOND_DENOM),
                     coin(balance_to_burn.u128(), NATIVE_FEE_DENOM),],
    });

    // Burn the tokens and response
    let mut res = Response::new().add_attribute("action", "clawback");

    res = res
        .add_message(clawback_msg)
        .add_attributes(vec![
            attr("amount", balance_to_burn),
        ]);

    Ok(res)
}

pub fn execute_pause(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    if info.sender != cfg.owner {
        return Err(ContractError::Unauthorized {});
    }

    PAUSED.save(deps.storage, &true)?;
    Ok(Response::new().add_attributes(vec![attr("action", "pause"), attr("paused", "true")]))
}

pub fn execute_resume(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // authorize owner
    let cfg = CONFIG.load(deps.storage)?;
    if info.sender != cfg.owner {
        return Err(ContractError::Unauthorized {});
    }

    let is_paused = PAUSED.load(deps.storage)?;
    if !is_paused {
        return Err(ContractError::NotPaused {});
    }

    PAUSED.save(deps.storage, &false)?;
    Ok(Response::new().add_attributes(vec![attr("action", "resume"), attr("paused", "false")]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::MerkleRoot {} => to_json_binary(&query_merkle_root(deps)?),
        QueryMsg::IsClaimed { address } => {
            to_json_binary(&query_is_claimed(deps, address)?)
        }
        QueryMsg::TotalClaimed {} => to_json_binary(&query_total_claimed(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: Some(cfg.owner.to_string()),
        claim_msg_plaintext: cfg.claim_msg_plaintext.to_string(),
    })
}

pub fn query_merkle_root(deps: Deps) -> StdResult<MerkleRootResponse> {
    let merkle_root = MERKLE_ROOT.load(deps.storage)?;
    let total_amount = AMOUNT.load(deps.storage)?;

    let resp = MerkleRootResponse {
        merkle_root,
        total_amount,
    };

    Ok(resp)
}

pub fn query_is_claimed(deps: Deps, address: String) -> StdResult<IsClaimedResponse> {
    let is_claimed = CLAIM.may_load(deps.storage, address)?.unwrap_or(false);
    let resp = IsClaimedResponse { is_claimed };

    Ok(resp)
}

pub fn query_total_claimed(deps: Deps) -> StdResult<TotalClaimedResponse> {
    let total_claimed = AMOUNT_CLAIMED.load(deps.storage)?;
    let resp = TotalClaimedResponse { total_claimed };

    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version = get_contract_version(deps.storage)?;
    if version.contract != CONTRACT_NAME {
        return Err(ContractError::CannotMigrate {
            previous_contract: version.contract,
        });
    }
    Ok(Response::default())
}

// src: https://github.com/public-awesome/launchpad/blob/main/contracts/sg-eth-airdrop/src/claim_airdrop.rs#L85
mod validation {
    use super::*;
    use cosmwasm_std::StdError;
    use ethereum_verify::verify_ethereum_text;

    use crate::state::Config;

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
        eth_pubkey: String,
        eth_sig: String,
        config: Config,
    ) -> Result<(), ContractError> {
        validate_eth_sig(deps, info, eth_pubkey.clone(), eth_sig, config)?;
        Ok(())
    }

    fn validate_eth_sig(
        deps: &DepsMut,
        info: MessageInfo,
        eth_pubkey: String,
        eth_sig: String,
        config: Config,
    ) -> Result<(), ContractError> {
        let valid_eth_sig =
            validate_ethereum_text(deps, info, &config, eth_sig, eth_pubkey.clone())?;
        match valid_eth_sig {
            true => Ok(()),
            false => Err(ContractError::AddressNotEligible {
                eth_pubkey: eth_pubkey,
            }),
        }
    }

    pub fn validate_ethereum_text(
        deps: &DepsMut,
        info: MessageInfo,
        config: &Config,
        eth_sig: String,
        eth_pubkey: String,
    ) -> StdResult<bool> {
        let plaintext_msg = compute_plaintext_msg(config, info);
        match hex::decode(eth_sig.clone()) {
            Ok(eth_sig_hex) => {
                verify_ethereum_text(deps.as_ref(), &plaintext_msg, &eth_sig_hex, &eth_pubkey)
            }
            Err(_) => Err(StdError::InvalidHex {
                msg: format!("Could not decode {eth_sig}"),
            }),
        }
    }
}
