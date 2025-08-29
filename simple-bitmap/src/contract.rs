use std::str::FromStr;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:staking-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    match msg {
        InstantiateMsg::Default { x_size, y_size } => {
            let string_size = (x_size as u32) * (y_size as u32) * 6;
            let state = crate::state::State {
                x_size,
                y_size,
                z_values: String::from_str(&"0".repeat(string_size as usize)).unwrap(),
            };
            STATE.save(deps.storage, &state)?;
            Ok(Response::new()
                .add_attribute("method", "instantiate")
                .add_attribute("x_size", x_size.to_string())
                .add_attribute("y_size", y_size.to_string()))
        }
        InstantiateMsg::WithString { x_size, y_size, z_values } => {
            let expected_size = (x_size as u32) * (y_size as u32) * 6;
            if z_values.len() != expected_size as usize {
                return Err(ContractError::InvalidZValue {});
            }
            let state = crate::state::State {
                x_size,
                y_size,
                z_values,
            };
            STATE.save(deps.storage, &state)?;
            Ok(Response::new()
                .add_attribute("method", "instantiate_with_string")
                .add_attribute("x_size", x_size.to_string())
                .add_attribute("y_size", y_size.to_string()))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Set { x, y, z } => execute::set(deps, x, y, z),
    }
}

pub mod execute {
    use super::*;

    pub fn set(deps: DepsMut, x: u8, y: u8, z: String) -> Result<Response, ContractError> {
        if x > STATE.load(deps.storage)?.x_size || y > STATE.load(deps.storage)?.y_size {
            return Err(ContractError::IndexOutOfBounds {});
        }
        let new_value_length = z.len();
        if new_value_length != 6 {
            return Err(ContractError::InvalidZValue {});
        }
        STATE.update(deps.storage, |mut state |-> Result<_, ContractError>{
            let y_offset= (y as u32) * (state.x_size as u32) * 6;
            let x_offset  = (x as u32) * 6;
            let start = (y_offset + x_offset) as usize;
            let end = (start + 6) as usize;
            state.z_values.replace_range(start..end, &z);
            Ok(state)
        })?;

        Ok(Response::new()
            .add_attribute("action", "set")
            .add_attribute("x", x.to_string())
            .add_attribute("y", y.to_string())
            .add_attribute("z", z.to_string()))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPoint {x, y} => to_json_binary(&query::get_point(deps, x, y)?),
        QueryMsg::GetGrid {} => to_json_binary(&query::get_grid(deps)?),
    }
}

pub mod query {
    use super::*;
    use crate::msg::{GetPointResponse,GetGridResponse};
    use crate::state::STATE;

    pub fn get_point(deps: Deps, x: u8, y: u8) -> StdResult<GetPointResponse> {
        let state = STATE.load(deps.storage)?;
        let y_offset= (y as u32) * (state.x_size as u32) * 6;
        let x_offset  = (x as u32) * 6;
        let start = (y_offset + x_offset) as usize;
        let end = (start + 6) as usize;
        let point = state.z_values[start..end].to_string();
        Ok(GetPointResponse { point })
    }

    pub fn get_grid(deps: Deps) -> StdResult<GetGridResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetGridResponse {
            x_size: state.x_size,
            y_size: state.y_size,
            z_values: state.z_values,
        })
    }
}

