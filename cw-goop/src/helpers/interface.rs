use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_binary, Addr, StdResult, WasmMsg, CosmosMsg,  QuerierWrapper, QueryRequest,  WasmQuery};

use crate::{
    msg::{ConfigResponse, QueryMsg,ExecuteMsg},
    state::Config,
};

/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CwGoopContract(pub Addr);

impl CwGoopContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn includes(&self, querier: &QuerierWrapper, member: String) -> StdResult<bool> {
        let includes: bool = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&QueryMsg::HasMember { member })?,
        }))?;
        Ok(includes)
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }

    pub fn per_address_limit(&self, querier: &QuerierWrapper) -> StdResult<u32> {
        let per_address_limit: u32 = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&QueryMsg::PerAddressLimit {})?,
        }))?;
        Ok(per_address_limit)
    }
}
