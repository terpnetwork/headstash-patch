use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_binary,CosmosMsg, Addr, StdResult, WasmMsg,QueryRequest, QuerierWrapper, WasmQuery};

use crate::msg::{ExecuteMsg, QueryMsg};

/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CwGoopContract(pub Addr);

impl CwGoopContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
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
    pub fn get_headstash_amount(&self, querier: &QuerierWrapper, address: String) -> StdResult<u128> {
        let get_headstash_amount: u128 = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_binary(&QueryMsg::GetHeadstashAmount{ address })?,
        }))?;
        Ok(get_headstash_amount)
    }
}
