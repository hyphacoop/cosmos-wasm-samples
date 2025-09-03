use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::Item;

use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
	pub x_size: u8,
	pub y_size: u8,
	pub z_values: String,
	pub recipient: String,
	pub supply_base_fee: u128,
	pub supply_fee_factor: u8,
	pub update_base_fee: u128,
	pub update_fee_factor: u8,
	pub fee_denom: String,
	pub set_points: Vec<(u8, u8)>,
	pub update_counts: BTreeMap<String, u32>, // key: "x_y"
}

pub const STATE: Item<State> = Item::new("state");

