use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Decimal};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Delegate {
        validator: String,
        amount: Coin,
    },
    Vote {
        proposal_id: u64,
        option: VoteOption,
    },
    WeightedVote {
        proposal_id: u64,
        options: Vec<WeightedVoteOption>,
    },
}

#[cw_serde]
pub enum VoteOption {
    Yes,
    No,
    Abstain,
    NoWithVeto,
}

#[cw_serde]
pub struct WeightedVoteOption {
    pub option: VoteOption,
    pub weight: Decimal,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // This contract doesn't have any queries yet
    #[returns(InfoResponse)]
    GetInfo {},
}

#[cw_serde]
pub struct InfoResponse {
    pub info: String,
}
