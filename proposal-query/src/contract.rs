use cosmos_sdk_proto::cosmos::gov::v1::{ProposalStatus, QueryProposalRequest, QueryProposalResponse};
use cosmos_sdk_proto::prost::Message;
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{InstantiateMsg, ProposalResponse, QueryMsg};

const CONTRACT_NAME: &str = "crates.io:proposal-query";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const PROPOSAL_QUERY_PATH: &str = "/cosmos.gov.v1.Query/Proposal";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Proposal { proposal_id } => {
            to_json_binary(&query_proposal(deps, proposal_id)?)
        }
    }
}

fn query_proposal(deps: Deps, proposal_id: u64) -> StdResult<ProposalResponse> {
    let request = QueryProposalRequest { proposal_id };
    let request_bytes = Binary::new(request.encode_to_vec());

    let response_bytes = deps
        .querier
        .query_grpc(PROPOSAL_QUERY_PATH.to_string(), request_bytes)?;

    let response = QueryProposalResponse::decode(response_bytes.as_slice())
        .map_err(|e| StdError::parse_err("QueryProposalResponse", e))?;

    let proposal = response
        .proposal
        .ok_or_else(|| StdError::generic_err("chain returned no proposal"))?;

    let status = ProposalStatus::try_from(proposal.status)
        .unwrap_or(ProposalStatus::Unspecified)
        .as_str_name()
        .to_string();

    let submit_time = proposal.submit_time.map(|t| t.seconds).unwrap_or_default();
    let voting_end_time = proposal
        .voting_end_time
        .map(|t| t.seconds)
        .unwrap_or_default();

    // final_tally_result is only populated once a proposal's voting period has ended.
    let (yes_count, abstain_count, no_count, no_with_veto_count) =
        match proposal.final_tally_result {
            Some(tally) => (
                tally.yes_count,
                tally.abstain_count,
                tally.no_count,
                tally.no_with_veto_count,
            ),
            None => (String::new(), String::new(), String::new(), String::new()),
        };

    Ok(ProposalResponse {
        id: proposal.id,
        title: proposal.title,
        summary: proposal.summary,
        status,
        proposer: proposal.proposer,
        submit_time,
        voting_end_time,
        yes_count,
        abstain_count,
        no_count,
        no_with_veto_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmos_sdk_proto::cosmos::gov::v1::TallyResult;
    use cosmos_sdk_proto::Timestamp;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env, MockApi, MockStorage};
    use cosmwasm_std::{
        from_json, Addr, ContractResult, Empty, GrpcQuery, OwnedDeps, QuerierResult, QueryRequest,
        Querier, SystemError, SystemResult,
    };
    use std::marker::PhantomData;

    /// A minimal test querier that only knows how to answer the one gRPC path this
    /// contract cares about; everything else is rejected the same way a real chain
    /// would reject an un-allowlisted gRPC query.
    struct GrpcMockQuerier;

    impl Querier for GrpcMockQuerier {
        fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
            let request: QueryRequest<Empty> = match from_json(bin_request) {
                Ok(v) => v,
                Err(e) => {
                    return SystemResult::Err(SystemError::InvalidRequest {
                        error: e.to_string(),
                        request: bin_request.into(),
                    })
                }
            };
            match request {
                QueryRequest::Grpc(GrpcQuery { path, .. }) if path == PROPOSAL_QUERY_PATH => {
                    let response = mock_grpc_proposal_response();
                    SystemResult::Ok(ContractResult::Ok(Binary::new(response.encode_to_vec())))
                }
                _ => SystemResult::Err(SystemError::UnsupportedRequest {
                    kind: "only the proposal gRPC query is mocked".to_string(),
                }),
            }
        }
    }

    fn mock_grpc_deps() -> OwnedDeps<MockStorage, MockApi, GrpcMockQuerier, Empty> {
        OwnedDeps {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: GrpcMockQuerier,
            custom_query_type: PhantomData,
        }
    }

    fn mock_grpc_proposal_response() -> QueryProposalResponse {
        QueryProposalResponse {
            proposal: Some(cosmos_sdk_proto::cosmos::gov::v1::Proposal {
                id: 42,
                messages: vec![],
                status: ProposalStatus::VotingPeriod as i32,
                final_tally_result: Some(TallyResult {
                    yes_count: "1000000".to_string(),
                    abstain_count: "0".to_string(),
                    no_count: "0".to_string(),
                    no_with_veto_count: "0".to_string(),
                }),
                submit_time: Some(Timestamp {
                    seconds: 1_700_000_000,
                    nanos: 0,
                }),
                deposit_end_time: None,
                total_deposit: vec![],
                voting_start_time: Some(Timestamp {
                    seconds: 1_700_000_100,
                    nanos: 0,
                }),
                voting_end_time: Some(Timestamp {
                    seconds: 1_700_100_000,
                    nanos: 0,
                }),
                metadata: String::new(),
                title: "Test Proposal".to_string(),
                summary: "A proposal used for testing".to_string(),
                proposer: "cosmos1abc".to_string(),
                expedited: false,
                failed_reason: String::new(),
            }),
        }
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn query_proposal_decodes_grpc_response() {
        let deps = mock_grpc_deps();

        let res = query_proposal(deps.as_ref(), 42).unwrap();
        assert_eq!(res.id, 42);
        assert_eq!(res.title, "Test Proposal");
        assert_eq!(res.status, "PROPOSAL_STATUS_VOTING_PERIOD");
        assert_eq!(res.proposer, "cosmos1abc");
        assert_eq!(res.submit_time, 1_700_000_000);
        assert_eq!(res.voting_end_time, 1_700_100_000);
        assert_eq!(res.yes_count, "1000000");
    }
}
