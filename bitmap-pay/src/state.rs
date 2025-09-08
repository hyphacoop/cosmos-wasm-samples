use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
	pub x_size: u8,
	pub y_size: u8,
	pub z_values: String,
	pub recipient: String,
	pub supply_base_fee: u128,
	pub supply_fee_factor: u128,
	pub update_base_fee: u128,
	pub update_fee_factor: u128,
	pub fee_factor_scale: u128,
	pub fee_denom: String,
	pub set_points: Vec<u8>, // bitfield: each bit represents a set point
	pub update_counts: Vec<u8>, // each byte is the update count for a point
}

pub const STATE: Item<State> = Item::new("state");

