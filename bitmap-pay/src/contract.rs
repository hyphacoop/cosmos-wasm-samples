use std::vec::Vec;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::STATE;

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
    let string_size = (msg.x_size as u32) * (msg.y_size as u32) * 6;
    let z_values = match msg.z_values {
        Some(ref z) => {
            if z.len() != string_size as usize {
                return Err(ContractError::InvalidZValue {});
            }
            z.clone()
        }
        None => "0".repeat(string_size as usize),
    };
    let state = crate::state::State {
        x_size: msg.x_size,
        y_size: msg.y_size,
        z_values,
        recipient: msg.recipient,
        supply_base_fee: msg.supply_base_fee,
        supply_fee_factor: msg.supply_fee_factor,
        update_base_fee: msg.update_base_fee,
        update_fee_factor: msg.update_fee_factor,
        fee_denom: msg.fee_denom.clone(),
        set_points: Vec::new(),
        update_counts: std::collections::BTreeMap::new(),
    };
    STATE.save(deps.storage, &state)?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("x_size", msg.x_size.to_string())
        .add_attribute("y_size", msg.y_size.to_string())
        .add_attribute("supply_base_fee", msg.supply_base_fee.to_string())
        .add_attribute("supply_fee_factor", msg.supply_fee_factor.to_string())
        .add_attribute("update_base_fee", msg.update_base_fee.to_string())
        .add_attribute("update_fee_factor", msg.update_fee_factor.to_string())
        .add_attribute("fee_denom", msg.fee_denom.clone())
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Set { x, y, z } => execute::set(deps, x, y, z, info),
    }
}

pub mod execute {
    use super::*;

    use cosmwasm_std::{BankMsg, MessageInfo};

    // Exponential bonding curve: cost = base * e^(factor * num_points_set)
    pub fn bonding_curve(base: u128, factor: f64, num_set: usize) -> u128 {
        let base_f = base as f64;
        let num_set_f = num_set as f64;
        let exp = (factor * num_set_f).exp();
        let result = base_f * exp;
        result.round() as u128
    }

    pub fn set(
        deps: DepsMut,
        x: u8,
        y: u8,
        z: String,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let mut state = STATE.load(deps.storage)?;
        if x > state.x_size || y > state.y_size {
            return Err(ContractError::IndexOutOfBounds {});
        }
        let new_value_length = z.len();
        if new_value_length != 6 {
            return Err(ContractError::InvalidZValue {});
        }

        let point = (x, y);
        let key = format!("{}_{}", x, y);
        let already_set = state.set_points.contains(&point);
        let update_count = *state.update_counts.get(&key).unwrap_or(&0);
        let supply_curve_cost = bonding_curve(state.supply_base_fee, state.supply_fee_factor as f64 / 100.0, state.set_points.len());
        let update_curve_cost= bonding_curve(state.update_base_fee, state.update_fee_factor as f64 / 100.0, update_count as usize);
        let set_point_cost = supply_curve_cost + update_curve_cost;
            let sent = info
                .funds
                .iter()
                .find(|c| c.denom == state.fee_denom)
                .map(|c| c.amount.u128())
                .unwrap_or(0);
        if sent < set_point_cost {
            return Err(ContractError::InsufficientFunds {});
        }
        // Transfer only the required cost to recipient
        let bank_msg = BankMsg::Send {
            to_address: state.recipient.clone(),
            amount: vec![cosmwasm_std::Coin {
                denom: state.fee_denom.clone(),
                amount: cosmwasm_std::Uint128::new(set_point_cost),
            }],
        };
        // Update grid, set_points, and update_counts
        let y_offset = (y as u32) * (state.x_size as u32) * 6;
        let x_offset = (x as u32) * 6;
        let start = (y_offset + x_offset) as usize;
        let end = (start + 6) as usize;
        state.z_values.replace_range(start..end, &z);

        if !already_set {
            state.set_points.push(point);
        }
        state.update_counts.insert(key, update_count + 1);
        STATE.save(deps.storage, &state)?;
        Ok(Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "set_point")
            .add_attribute("x", x.to_string())
            .add_attribute("y", y.to_string())
            .add_attribute("z", z.to_string())
            .add_attribute("from", info.sender)
            .add_attribute("cost", set_point_cost.to_string())
            .add_attribute("recipient", state.recipient)
            .add_attribute("already_set", already_set.to_string())
            .add_attribute("update_count", (update_count + 1).to_string()))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPoint { x, y } => to_json_binary(&query::get_point(deps, x, y)?),
        QueryMsg::GetGrid {} => to_json_binary(&query::get_grid(deps)?),
        QueryMsg::GetCost { x, y } => to_json_binary(&query::get_cost(deps, x, y)?),
        QueryMsg::GetParams {} => to_json_binary(&query::get_params(deps)?),
    }
}

pub mod query {
    use super::*;
    use crate::msg::{GetCostResponse, GetGridResponse, GetPointResponse, GetParamsResponse};
    use crate::state::STATE;

    pub fn get_point(deps: Deps, x: u8, y: u8) -> StdResult<GetPointResponse> {
        let state = STATE.load(deps.storage)?;
        let y_offset = (y as u32) * (state.x_size as u32) * 6;
        let x_offset = (x as u32) * 6;
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

    pub fn get_cost(deps: Deps, x: u8, y: u8) -> StdResult<GetCostResponse> {
        let state = STATE.load(deps.storage)?;
        let key = format!("{}_{}", x, y);
        let update_count = *state.update_counts.get(&key).unwrap_or(&0);
        // let supply_factor = state.supply_fee_factor as f64 / 100.0;
        let supply_curve_cost = super::execute::bonding_curve(state.supply_base_fee, state.supply_fee_factor as f64 / 100.0, state.set_points.len());  // base=50, factor=0.5
        let update_curve_cost= super::execute::bonding_curve(state.update_base_fee, state.update_fee_factor as f64 / 100.0, update_count as usize);
        let set_point_cost = supply_curve_cost + update_curve_cost;
        Ok(GetCostResponse { cost: set_point_cost })
    }

    pub fn get_params(deps: Deps) -> StdResult<GetParamsResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetParamsResponse {
            supply_base_fee: state.supply_base_fee,
            supply_fee_factor: state.supply_fee_factor,
            update_base_fee: state.update_base_fee,
            update_fee_factor: state.update_fee_factor,
            fee_denom: state.fee_denom.clone(),
        })
    }
    
}
