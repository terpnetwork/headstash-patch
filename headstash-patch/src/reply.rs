#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::state::CONFIG;
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, Reply};
use cw_utils::{parse_reply_instantiate_data, MsgInstantiateContractResponse, ParseReplyError};
use sg_std::Response;

const INIT_HEADSTASH_GROUP_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != INIT_HEADSTASH_GROUP_ID {
        return Err(ContractError::InvalidReplyID {});
    }
    let reply = parse_reply_instantiate_data(msg);
    match_reply(deps, reply)
}

fn match_reply(
    deps: DepsMut,
    reply: Result<MsgInstantiateContractResponse, ParseReplyError>,
) -> Result<Response, ContractError> {
    match reply {
        Ok(res) => {
            let headstash_group_address = &res.contract_address;
            let mut config = CONFIG.load(deps.storage)?;
            config.headstash_group_address = Some(headstash_group_address.to_string());
            CONFIG.save(deps.storage, &config)?;

            Ok(Response::default()
                .add_attribute("action", "init_headstash_group_reply")
                .add_attribute("headstash_group_address", headstash_group_address))
        }
        Err(_) => Err(ContractError::ReplyOnSuccess {}),
    }
}