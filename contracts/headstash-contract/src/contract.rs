#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128, WasmMsg, Coin , SubMsg, BankMsg, 
};
use cw2::{get_contract_version, set_contract_version};
// use cw20_vesting::ExecuteMsg as Cw20ExecuteMsg;
use crate::contract::validation::validate_claim;
use cw_utils::{Expiration, Scheduled};
use sha2::Digest;
use std::convert::TryInto;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, IsClaimedResponse, LatestStageResponse,
    MerkleRootResponse, MigrateMsg, QueryMsg, TotalClaimedResponse,
};
use crate::state::{
    Config, StageAmounts, NATIVE_FEE_DENOM, NATIVE_BOND_DENOM, StageDetails, CLAIM, CONFIG, LATEST_STAGE, MERKLE_ROOT, STAGE_AMOUNTS,
    STAGE_DETAILS,
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
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = msg
        .owner
        .map_or(Ok(info.sender), |o| deps.api.addr_validate(&o))?;

    let config = Config {
        owner: Some(owner),
        claim_msg_plaintext: msg.claim_msg_plaintext,
    };
    CONFIG.save(deps.storage, &config)?;

    let stage = 0;
    LATEST_STAGE.save(deps.storage, &stage)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig { new_owner } => execute_update_config(deps, env, info, new_owner),
        ExecuteMsg::RegisterMerkleRoot {
            merkle_root,
            expiration,
            start,
            total_amount,
        } => execute_register_merkle_root(
            deps,
            env,
            info,
            merkle_root,
            expiration,
            start,
            total_amount,
        ),
        ExecuteMsg::Claim {
            stage,
            amount,
            proof,
            eth_pubkey,
            eth_sig,
        } => execute_claim(deps, env, info, stage, eth_pubkey, eth_sig, amount, proof ),
        // ExecuteMsg::Burn { stage } => execute_burn(deps, env, info, stage),
        ExecuteMsg::ClawBack { stage, recipient } => {
            execute_clawback(deps, env, info, stage, Some(recipient))
        }
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_owner: Option<String>,
) -> Result<Response, ContractError> {
    // authorize owner
    let cfg = CONFIG.load(deps.storage)?;
    let owner = cfg.owner.ok_or(ContractError::Unauthorized {})?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    // if owner some validated to addr, otherwise set to none
    let mut tmp_owner = None;
    if let Some(addr) = new_owner {
        tmp_owner = Some(deps.api.addr_validate(&addr)?)
    }

    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.owner = tmp_owner;
        Ok(exists)
    })?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

#[allow(clippy::too_many_arguments)]
pub fn execute_register_merkle_root(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    merkle_root: String,
    expiration: Expiration,
    start: Scheduled,
    total_amount: Uint128,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    // if owner set validate, otherwise unauthorized
    let owner = cfg.owner.ok_or(ContractError::Unauthorized {})?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }


    // check merkle root length
    let mut root_buf: [u8; 32] = [0; 32];
    hex::decode_to_slice(&merkle_root, &mut root_buf)?;

    let stage = LATEST_STAGE.update(deps.storage, |stage| -> StdResult<_> { Ok(stage + 1) })?;

    MERKLE_ROOT.save(deps.storage, stage, &merkle_root)?;
    LATEST_STAGE.save(deps.storage, &stage)?;

    let details = StageDetails {
        expiration,
        start,
    };
    STAGE_DETAILS.save(deps.storage, stage, &details)?;

    let amounts = StageAmounts {
        total: total_amount,
        claimed: Uint128::zero(),
    };
    STAGE_AMOUNTS.save(deps.storage, stage, &amounts)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "register_merkle_root"),
        attr("stage", stage.to_string()),
        attr("merkle_root", merkle_root),
        attr("total_amount", total_amount),
    ]))
}

pub fn execute_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stage: u8,
    eth_pubkey: String,
    eth_sig: String,
    amount: Uint128,
    proof: Vec<String>,
) -> Result<Response, ContractError> {
    let StageDetails {
        expiration,
        start,
    } = STAGE_DETAILS.load(deps.storage, stage)?;

    // airdrop begun
    if !start.is_triggered(&env.block) {
        return Err(ContractError::StageNotBegun { stage, start });
    }
    // not expired
    if expiration.is_expired(&env.block) {
        return Err(ContractError::StageExpired { stage, expiration });
    }

    // verify not claimed
    let claimed = CLAIM.may_load(deps.storage, (&info.sender, stage))?;
    if claimed.is_some() {
        return Err(ContractError::Claimed {});
    }

    let config = CONFIG.load(deps.storage)?;
    let merkle_root = MERKLE_ROOT.load(deps.storage, stage)?;
    validate_claim( &deps,
        info.clone(),
        eth_pubkey.clone(),
        eth_sig,
        config.clone(),
    )?;

    let user_input = format!("{}{}", info.sender, amount);
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

    // record the claim (individually and total)
    CLAIM.save(deps.storage, (&info.sender, stage), &true)?;
    STAGE_AMOUNTS.update::<_, ContractError>(deps.storage, stage, |old| {
        let mut val = old.ok_or_else(|| StdError::not_found("stage_amounts"))?;
        // TODO: ensure we cannot claim more than total
        val.claimed += amount;
        Ok(val)
    })?;

    let bank_msg = SubMsg::new(BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![
            Coin {
                denom: NATIVE_BOND_DENOM.to_string(),
                amount: Uint128::new(12345),
            },
            Coin {
                denom: NATIVE_FEE_DENOM.to_string(),
                amount: Uint128::new(67890),
            },
        ],
    });

    let res = Response::new()
        .add_submessage(bank_msg)
        .add_attributes(vec![
            attr("action", "claim"),
            attr("stage", stage.to_string()),
            attr("address", info.sender),
            attr("amount", amount),
        ]);
    Ok(res)
}


pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stage: u8,
) -> Result<Response, ContractError> {
    execute_clawback(deps, env, info, stage, None)
}

pub fn execute_clawback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stage: u8,
    recipient: Option<String>,
) -> Result<Response, ContractError> {
    // authorize owner
    let cfg = CONFIG.load(deps.storage)?;
    let owner = cfg.owner.ok_or(ContractError::Unauthorized {})?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }

    // make sure is expired
    let StageDetails {
        expiration,
        ..
    } = STAGE_DETAILS.load(deps.storage, stage)?;
    if !expiration.is_expired(&env.block) {
        return Err(ContractError::StageNotExpired { stage, expiration });
    }

    // Get total amount per stage and total claimed
    let StageAmounts { total, claimed } = STAGE_AMOUNTS.load(deps.storage, stage)?;

    // Get balance
    let balance_to_burn = total.checked_sub(claimed)?;

    // let msg = Cw20ExecuteMsg::Burn {  // to-do burn native-denom tokens
    //     amount: balance_to_burn,
    // };

    // Burn the tokens and response
    let mut res = Response::new().add_attribute("action", "burn");

    res = res
        // .add_message(msg)
        .add_attributes(vec![
            attr("stage", stage.to_string()),
            attr("address", info.sender),
            attr("amount", balance_to_burn),
        ]);

    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::MerkleRoot { stage } => to_binary(&query_merkle_root(deps, stage)?),
        QueryMsg::LatestStage {} => to_binary(&query_latest_stage(deps)?),
        QueryMsg::IsClaimed { stage, address } => {
            to_binary(&query_is_claimed(deps, stage, address)?)
        }
        QueryMsg::TotalClaimed { stage } => to_binary(&query_total_claimed(deps, stage)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: cfg.owner.map(|o| o.to_string()),
        claim_msg_plaintext: cfg.claim_msg_plaintext.to_string(),
    })
}

pub fn query_merkle_root(deps: Deps, stage: u8) -> StdResult<MerkleRootResponse> {
    let merkle_root = MERKLE_ROOT.load(deps.storage, stage)?;
    let StageDetails {
        expiration,
        start,
    } = STAGE_DETAILS.load(deps.storage, stage)?;
    let StageAmounts { total, claimed } = STAGE_AMOUNTS.load(deps.storage, stage)?;

    let resp = MerkleRootResponse {
        stage,
        merkle_root,
        expiration,
        start,
        total_amount: total,
        claimed_amount: claimed,
    };

    Ok(resp)
}

pub fn query_latest_stage(deps: Deps) -> StdResult<LatestStageResponse> {
    let latest_stage = LATEST_STAGE.load(deps.storage)?;
    let resp = LatestStageResponse { latest_stage };

    Ok(resp)
}

pub fn query_is_claimed(deps: Deps, stage: u8, address: String) -> StdResult<IsClaimedResponse> {
    let key: (&Addr, u8) = (&deps.api.addr_validate(&address)?, stage);
    let is_claimed = CLAIM.may_load(deps.storage, key)?.unwrap_or(false);
    let resp = IsClaimedResponse { is_claimed };

    Ok(resp)
}

pub fn query_total_claimed(deps: Deps, stage: u8) -> StdResult<TotalClaimedResponse> {
    let StageAmounts { total, claimed } = STAGE_AMOUNTS.load(deps.storage, stage)?;
    let resp = TotalClaimedResponse { total, claimed };

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

mod validation {
    use super::*;
    use cosmwasm_std::StdError;
    use ethereum_verify::verify_ethereum_text;

    use crate::{
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




#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, from_slice, CosmosMsg, SubMsg};
    use serde::Deserialize;

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            owner: Some("owner0000".to_string()),
            claim_msg_plaintext: "{address}".to_string(),
        };

        let env = mock_env();
        let info = mock_info("{address}", &[]);

        // we can just call .unwrap() to assert this was a success
        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        // it worked, let's query the state
        let res = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_binary(&res).unwrap();
        assert_eq!("owner0000", config.owner.unwrap().as_str());
        assert_eq!("{address}", config.claim_msg_plaintext.as_str());

        let res = query(deps.as_ref(), env, QueryMsg::LatestStage {}).unwrap();
        let latest_stage: LatestStageResponse = from_binary(&res).unwrap();
        assert_eq!(0u8, latest_stage.latest_stage);
    }

    #[test]
    fn update_config() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            owner: None,
            claim_msg_plaintext: "{address}".to_string(),
        };

        let env = mock_env();
        let info = mock_info("owner0000", &[]);
        let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        // update owner
        let env = mock_env();
        let info = mock_info("owner0000", &[]);
        let msg = ExecuteMsg::UpdateConfig {
            new_owner: Some("owner0001".to_string()),
        };

        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_binary(&res).unwrap();
        assert_eq!("owner0001", config.owner.unwrap().as_str());

        // Unauthorized err
        let env = mock_env();
        let info = mock_info("owner0000", &[]);
        let msg = ExecuteMsg::UpdateConfig { new_owner: None };

        let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});
    }

    #[test]
    fn register_merkle_root() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            owner: Some("owner0000".to_string()),
            claim_msg_plaintext: "{address}".to_string(),
        };

        let env = mock_env();
        let info = mock_info("addr0000", &[]);
        let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        // register new merkle root
        let env = mock_env();
        let info = mock_info("owner0000", &[]);
        let msg = ExecuteMsg::default_merkle_root(
            "634de21cde1044f41d90373733b0f0fb1c1c71f9652b905cdf159e73c4cf0d37",
        );

        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        assert_eq!(
            res.attributes,
            vec![
                attr("action", "register_merkle_root"),
                attr("stage", "1"),
                attr(
                    "merkle_root",
                    "634de21cde1044f41d90373733b0f0fb1c1c71f9652b905cdf159e73c4cf0d37"
                ),
                attr("total_amount", ExecuteMsg::default_total().to_string())
            ]
        );

        let res = query(deps.as_ref(), env.clone(), QueryMsg::LatestStage {}).unwrap();
        let latest_stage: LatestStageResponse = from_binary(&res).unwrap();
        assert_eq!(1u8, latest_stage.latest_stage);

        let res = query(
            deps.as_ref(),
            env,
            QueryMsg::MerkleRoot {
                stage: latest_stage.latest_stage,
            },
        )
        .unwrap();
        let merkle_root: MerkleRootResponse = from_binary(&res).unwrap();
        assert_eq!(
            "634de21cde1044f41d90373733b0f0fb1c1c71f9652b905cdf159e73c4cf0d37".to_string(),
            merkle_root.merkle_root
        );
    }

    const TEST_DATA_1: &[u8] = include_bytes!("../testdata/airdrop_stage_1_test_data.json");
    const TEST_DATA_2: &[u8] = include_bytes!("../testdata/airdrop_stage_2_test_data.json");

    #[derive(Deserialize, Debug)]
    struct Encoded {
        account: String,
        amount: Uint128,
        root: String,
        proofs: Vec<String>,
    }

    // #[test]
    // fn claim() {
    //     // Run test 1
    //     let mut deps = mock_dependencies();
    //     let test_data: Encoded = from_slice(TEST_DATA_1).unwrap();

    //     let msg = InstantiateMsg {
    //         owner: Some("owner0000".to_string()),
    //         cw20_token_address: "token0000".to_string(),
    //     };

    //     let env = mock_env();
    //     let info = mock_info("addr0000", &[]);
    //     let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    //     let env = mock_env();
    //     let info = mock_info("owner0000", &[]);
    //     let msg = ExecuteMsg::default_merkle_root(test_data.root);
    //     let _res = execute(deps.as_mut(), env, info, msg).unwrap();

    //     let msg = ExecuteMsg::Claim {
    //         amount: test_data.amount,
    //         stage: 1u8,
    //         proof: test_data.proofs,
    //     };

    //     let env = mock_env();
    //     let info = mock_info(test_data.account.as_str(), &[]);
    //     let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
    //     let expected = SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
    //         contract_addr: "token0000".to_string(),
    //         funds: vec![],
    //         msg: to_binary(&Cw20ExecuteMsg::Transfer {
    //             recipient: test_data.account.clone(),
    //             amount: test_data.amount,
    //         })
    //         .unwrap(),
    //     }));
    //     assert_eq!(res.messages, vec![expected]);

    //     assert_eq!(
    //         res.attributes,
    //         vec![
    //             attr("action", "claim"),
    //             attr("stage", "1"),
    //             attr("address", test_data.account.clone()),
    //             attr("amount", test_data.amount)
    //         ]
    //     );

    //     // Check total claimed on stage 1
    //     let claimed = query_total_claimed(deps.as_ref(), 1).unwrap();
    //     assert_eq!(claimed.claimed, test_data.amount);

    //     // Check address is claimed
    //     let is_claimed = query_is_claimed(deps.as_ref(), 1, test_data.account).unwrap();
    //     assert!(is_claimed.is_claimed);

    //     // check error on double claim
    //     let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    //     assert_eq!(res, ContractError::Claimed {});

    //     // Second test
    //     let test_data: Encoded = from_slice(TEST_DATA_2).unwrap();

    //     // register new drop
    //     let env = mock_env();
    //     let info = mock_info("owner0000", &[]);
    //     let msg = ExecuteMsg::default_merkle_root(test_data.root);
    //     let _res = execute(deps.as_mut(), env, info, msg).unwrap();

    //     // Claim next airdrop
    //     let msg = ExecuteMsg::Claim {
    //         amount: test_data.amount,
    //         stage: 2u8,
    //         proof: test_data.proofs,
    //     };

    //     let env = mock_env();
    //     let info = mock_info(test_data.account.as_str(), &[]);
    //     let res = execute(deps.as_mut(), env, info, msg).unwrap();
    //     let expected: SubMsg<_> = SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
    //         contract_addr: "token0000".to_string(),
    //         funds: vec![],
    //         msg: to_binary(&Cw20ExecuteMsg::Transfer {
    //             recipient: test_data.account.clone(),
    //             amount: test_data.amount,
    //         })
    //         .unwrap(),
    //     }));
    //     assert_eq!(res.messages, vec![expected]);

    //     assert_eq!(
    //         res.attributes,
    //         vec![
    //             attr("action", "claim"),
    //             attr("stage", "2"),
    //             attr("address", test_data.account),
    //             attr("amount", test_data.amount)
    //         ]
    //     );

    //     // Check total claimed on stage 2
    //     let claimed = query_total_claimed(deps.as_ref(), 2).unwrap();
    //     assert_eq!(claimed.claimed, test_data.amount);
    // }

    const TEST_DATA_1_MULTI: &[u8] =
        include_bytes!("../testdata/airdrop_stage_1_test_multi_data.json");

    #[derive(Deserialize, Debug)]
    struct Proof {
        account: String,
        amount: Uint128,
        proofs: Vec<String>,
    }

    #[derive(Deserialize, Debug)]
    struct MultipleData {
        total_amount: Uint128,
        total_claimed_amount: Uint128,
        root: String,
        accounts: Vec<Proof>,
    }

    // #[test]
    // fn multiple_claim() {
    //     // Run test 1
    //     let mut deps = mock_dependencies();
    //     let test_data: MultipleData = from_slice(TEST_DATA_1_MULTI).unwrap();

    //     let msg = InstantiateMsg {
    //         owner: Some("owner0000".to_string()),
    //         cw20_token_address: "token0000".to_string(),
    //     };

    //     let env = mock_env();
    //     let info = mock_info("addr0000", &[]);
    //     let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    //     let env = mock_env();
    //     let info = mock_info("owner0000", &[]);
    //     let msg = ExecuteMsg::register_merkle_root(
    //         test_data.root,
    //         test_data.total_amount.u128(),
    //         None,
    //         None,
    //         None,
    //     );
    //     let _res = execute(deps.as_mut(), env, info, msg).unwrap();

    //     // Loop accounts and claim
    //     for account in test_data.accounts.iter() {
    //         let msg = ExecuteMsg::Claim {
    //             amount: account.amount,
    //             stage: 1u8,
    //             proof: account.proofs.clone(),
    //         };

    //         let env = mock_env();
    //         let info = mock_info(account.account.as_str(), &[]);
    //         let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
    //         let expected = SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
    //             contract_addr: "token0000".to_string(),
    //             funds: vec![],
    //             msg: to_binary(&Cw20ExecuteMsg::Transfer {
    //                 recipient: account.account.clone(),
    //                 amount: account.amount,
    //             })
    //             .unwrap(),
    //         }));
    //         assert_eq!(res.messages, vec![expected]);

    //         assert_eq!(
    //             res.attributes,
    //             vec![
    //                 attr("action", "claim"),
    //                 attr("stage", "1"),
    //                 attr("address", account.account.clone()),
    //                 attr("amount", account.amount)
    //             ]
    //         );
    //     }

    //     // Check total claimed on stage 1
    //     let totals = query_total_claimed(deps.as_ref(), 1).unwrap();
    //     assert_eq!(totals.claimed, test_data.total_claimed_amount);
    //     assert_eq!(totals.total, test_data.total_amount);
    // }

    // Check expiration. Chain height in tests is 12345
    #[test]
    fn stage_expires() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            owner: Some("owner0000".to_string()),
            claim_msg_plaintext: "{address}".to_string(),
        };

        let env = mock_env();
        let info = mock_info("addr0000", &[]);
        let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        // can register merkle root
        let env = mock_env();
        let info = mock_info("owner0000", &[]);
        let msg = ExecuteMsg::register_merkle_root(
            "5d4f48f147cb6cb742b376dce5626b2a036f69faec10cd73631c791780e150fc",
            123000000,
            Some(Expiration::AtHeight(100)),
            None,
        );
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // can't claim expired
        let msg = ExecuteMsg::Claim {
            amount: Uint128::new(5),
            stage: 1u8,
            proof: vec![],
            eth_pubkey: "0x24".to_string(),
            eth_sig: "05XFb80".to_string(),
        };

        let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(
            res,
            ContractError::StageExpired {
                stage: 1,
                expiration: Expiration::AtHeight(100)
            }
        )
    }


    // #[test]
    // fn can_clawback() {
    //     let mut deps = mock_dependencies();
    //     let test_data: Encoded = from_slice(TEST_DATA_1).unwrap();

    //     let msg = InstantiateMsg {
    //         owner: Some("owner0000".to_string()),
    //         claim_msg_plaintext: "{address}".to_string(),
    //     };
    //     let info = mock_info("addr0000", &[]);
    //     instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     let info = mock_info("owner0000", &[]);
    //     let msg = ExecuteMsg::RegisterMerkleRoot {
    //         merkle_root: test_data.root,
    //         expiration: Expiration::AtHeight(12500),
    //         start: ExecuteMsg::default_start(),
    //         total_amount: Uint128::new(10000),
    //     };
    //     execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     let mut env = mock_env();
    //     env.block.height = 10000;

    //     // cannot yet clawback
    //     let msg = ExecuteMsg::ClawBack {
    //         stage: 1,
    //         recipient: "buddy".to_string(),
    //     };
    //     let info = mock_info("owner0000", &[]);
    //     let err = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap_err();
    //     assert_eq!(
    //         err,
    //         ContractError::StageNotExpired {
    //             stage: 1,
    //             expiration: Expiration::AtHeight(12500)
    //         }
    //     );

    //     // makes the stage expire
    //     env.block.height = 12501;

    //     // Can clawback after expired stage
    //     let res = execute(deps.as_mut(), env, info, msg).unwrap();

    //     let expected = SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
    //         contract_addr: "token0000".to_string(),
    //         funds: vec![],
    //         msg: to_binary(&Cw20ExecuteMsg::Transfer {
    //             recipient: "buddy".to_string(),
    //             amount: Uint128::new(10000),
    //         })
    //         .unwrap(),
    //     }));
    //     assert_eq!(res.messages, vec![expected]);

    //     assert_eq!(
    //         res.attributes,
    //         vec![
    //             attr("action", "clawback"),
    //             attr("recipient", "buddy"),
    //             attr("stage", "1"),
    //             attr("address", "owner0000"),
    //             attr("amount", Uint128::new(10000)),
    //         ]
    //     );
    // }


    #[test]
    fn stage_starts() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            owner: Some("owner0000".to_string()),
            claim_msg_plaintext: "{address}".to_string(),
        };

        let env = mock_env();
        let info = mock_info("addr0000", &[]);
        let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        // can register merkle root
        let env = mock_env();
        let info = mock_info("owner0000", &[]);
        let msg = ExecuteMsg::register_merkle_root(
            "5d4f48f147cb6cb742b376dce5626b2a036f69faec10cd73631c791780e150fc",
            123000,
            None,
            Some(Scheduled::AtHeight(200_000)),
        );
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // can't claim expired
        let msg = ExecuteMsg::Claim {
            amount: Uint128::new(5),
            stage: 1u8,
            proof: vec![],
            eth_pubkey: "0x2397cdefa601ECee59E4ab8bB13c756D9b34d157".to_string(),
            eth_sig: "78gyuvg".to_string(),

        };

        let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(
            res,
            ContractError::StageNotBegun {
                stage: 1,
                start: Scheduled::AtHeight(200_000)
            }
        )
    }

    #[test]
    fn owner_freeze() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            owner: Some("owner0000".to_string()),
            claim_msg_plaintext: "{address}".to_string(),
        };

        let env = mock_env();
        let info = mock_info("addr0000", &[]);
        let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        // can register merkle root
        let env = mock_env();
        let info = mock_info("owner0000", &[]);
        let msg = ExecuteMsg::default_merkle_root(
            "5d4f48f147cb6cb742b376dce5626b2a036f69faec10cd73631c791780e150fc",
        );
        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

        // can update owner
        let env = mock_env();
        let info = mock_info("owner0000", &[]);
        let msg = ExecuteMsg::UpdateConfig {
            new_owner: Some("owner0001".to_string()),
        };

        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // freeze contract
        let env = mock_env();
        let info = mock_info("owner0001", &[]);
        let msg = ExecuteMsg::UpdateConfig { new_owner: None };

        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // cannot register new drop
        let env = mock_env();
        let info = mock_info("owner0001", &[]);
        let msg = ExecuteMsg::default_merkle_root(
            "ebaa83c7eaf7467c378d2f37b5e46752d904d2d17acd380b24b02e3b398b3e5a",
        );
        let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});

        // cannot update config
        let env = mock_env();
        let info = mock_info("owner0001", &[]);
        let msg = ExecuteMsg::UpdateConfig {
            new_owner: Some("mememe".to_string()),
        };
        let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});
    }
}
