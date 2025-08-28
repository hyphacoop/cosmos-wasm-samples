use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// use cw_storage_plus::Map;

use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
	pub x_size: u8,
	pub y_size: u8,
	pub z_values: String,
}

pub const STATE: Item<State> = Item::new("state");

// The grid is represented as a map from (x, y) to point (z)
// pub const GRID: Map<(u32, u32), u32> = Map::new("grid");
