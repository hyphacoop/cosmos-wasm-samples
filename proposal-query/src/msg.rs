use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Queries a governance proposal via the chain's gRPC gov module
    /// (`/cosmos.gov.v1.Query/Proposal`) and returns the decoded result.
    #[returns(ProposalResponse)]
    Proposal { proposal_id: u64 },
}

#[cw_serde]
pub struct ProposalResponse {
    pub id: u64,
    pub title: String,
    pub summary: String,
    pub status: String,
    pub proposer: String,
    pub submit_time: i64,
    pub voting_end_time: i64,
    pub yes_count: String,
    pub abstain_count: String,
    pub no_count: String,
    pub no_with_veto_count: String,
}
