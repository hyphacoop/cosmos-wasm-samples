use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Queries a validator via the chain's gRPC staking module
    /// (`/cosmos.staking.v1beta1.Query/Validator`) and returns the decoded result.
    #[returns(ValidatorResponse)]
    Validator { validator_addr: String },
}

#[cw_serde]
pub struct ValidatorResponse {
    pub operator_address: String,
    pub moniker: String,
    pub jailed: bool,
    pub status: String,
    pub tokens: String,
    pub delegator_shares: String,
    pub commission_rate: String,
}
