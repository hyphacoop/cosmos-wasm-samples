#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_json};
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, GetPointResponse};

    #[test]
    fn set_and_query_grid() {
        let mut deps = mock_dependencies();

        // Instantiate contract
        let msg = InstantiateMsg { x_size: 5, y_size: 5 };
        let info = mock_info("creator", &coins(1000, "earth"));
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Set point at (2, 3) to 7
        let info = mock_info("user", &[]);
        let set_msg = ExecuteMsg::Set { x: 2, y: 3, z: 7 };
        let _res = execute(deps.as_mut(), mock_env(), info, set_msg).unwrap();

        // Query point at (2, 3)
        let query_msg = QueryMsg::GetPoint { x: 2, y: 3 };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let value: GetPointResponse = from_json(&res).unwrap();
        assert_eq!(value.point, Some(7));

        // Query point at (0, 0) (never set)
        let query_msg = QueryMsg::GetPoint { x: 0, y: 0 };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let value: GetPointResponse = from_json(&res).unwrap();
        assert_eq!(value.point, None);
    }
}
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
    let string_size= ((msg.x_size as u32) * (msg.y_size as u32) * 6) as usize;
    let state = crate::state::State {
        x_size: msg.x_size,
        y_size: msg.y_size,
        z_values: String::from_str(&"0".repeat(string_size)).unwrap(),
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("x_size", msg.x_size.to_string())
        .add_attribute("y_size", msg.y_size.to_string()))
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

