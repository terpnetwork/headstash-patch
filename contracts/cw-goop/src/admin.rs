use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, StdResult, Response};

use crate::{
   msg::{AdminResponse, CanExecuteResponse},
    state::ADMIN_LIST,
    ContractError,
};


pub fn can_execute(deps: &DepsMut, sender: Addr) -> Result<Addr, ContractError> {
    let cfg = ADMIN_LIST.load(deps.storage)?;
    let can = cfg.is_admin(&sender);
    if !can {
        return Err(ContractError::Unauthorized {});
    }
    Ok(sender)
}

pub fn execute_freeze(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut cfg = ADMIN_LIST.load(deps.storage)?;
    if !cfg.can_modify(info.sender.as_ref()) {
        Err(ContractError::Unauthorized {})
    } else {
        cfg.mutable = false;
        ADMIN_LIST.save(deps.storage, &cfg)?;

        let res = Response::new().add_attribute("action", "freeze");
        Ok(res)
    }
}

pub fn query_admin_list(deps: Deps) -> StdResult<AdminResponse> {
    let cfg = ADMIN_LIST.load(deps.storage)?;
    Ok(AdminResponse {
        admin: cfg.admin,
        mutable: cfg.mutable,
    })
}

pub fn query_can_execute(deps: Deps, sender: &str) -> StdResult<CanExecuteResponse> {
    let cfg = ADMIN_LIST.load(deps.storage)?;
    let can = cfg.is_admin(deps.api.addr_validate(sender)?);
    Ok(CanExecuteResponse { can_execute: can })
}
