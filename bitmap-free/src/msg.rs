
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub x_size: u8,
    pub y_size: u8,
    pub z_values: Option<String>,
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
}

#[cw_serde]
pub struct GetPointResponse {
    pub point: String,
}

#[cw_serde]
pub struct GetGridResponse {
    pub x_size: u8,
    pub y_size: u8,
    pub z_values: String,
}