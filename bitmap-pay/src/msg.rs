
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub x_size: u8,
    pub y_size: u8,
    pub z_values: Option<String>,
    pub recipient: String,
    pub supply_base_fee: u128,
    pub supply_fee_factor: u128,
    pub update_base_fee: u128,
    pub update_fee_factor: u128,
    pub fee_factor_scale: u128,
    pub fee_denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Set { x: u8, y: u8, z: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// GetPoint returns the point (z) at (x, y)
    #[returns(GetPointResponse)]
    GetPoint { x: u8, y: u8 },

    /// GetGrid returns the entire grid as a string
    #[returns(GetGridResponse)]
    GetGrid {},

    // GetCost returns the cost to set a point (x, y)
    #[returns(GetCostResponse)]
    GetCost { x: u8, y: u8 },

    // GetParams returns the curve parameters (base, factor)
    #[returns(GetParamsResponse)]
    GetParams {},
}

#[cw_serde]
pub struct GetCostResponse {
    pub cost: u128,
}

#[cw_serde]
pub struct GetPointResponse {
    pub point: String,
    pub is_set: bool,
    pub update_count: u8,
}

#[cw_serde]
pub struct GetGridResponse {
    pub x_size: u8,
    pub y_size: u8,
    pub z_values: String,
}

#[cw_serde]
pub struct GetParamsResponse {
    pub supply_base_fee: u128,
    pub supply_fee_factor: u128,
    pub update_base_fee: u128,
    pub update_fee_factor: u128,
    pub fee_factor_scale: u128,
    pub fee_denom: String,
}