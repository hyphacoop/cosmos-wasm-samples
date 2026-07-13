use cosmos_sdk_proto::cosmos::staking::v1beta1::{
    BondStatus, QueryValidatorRequest, QueryValidatorResponse,
};
use cosmos_sdk_proto::prost::Message;
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{InstantiateMsg, QueryMsg, ValidatorResponse};

const CONTRACT_NAME: &str = "crates.io:validator-query";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const VALIDATOR_QUERY_PATH: &str = "/cosmos.staking.v1beta1.Query/Validator";

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
        QueryMsg::Validator { validator_addr } => {
            to_json_binary(&query_validator(deps, validator_addr)?)
        }
    }
}

fn query_validator(deps: Deps, validator_addr: String) -> StdResult<ValidatorResponse> {
    let request = QueryValidatorRequest { validator_addr };
    let request_bytes = Binary::new(request.encode_to_vec());

    let response_bytes = deps
        .querier
        .query_grpc(VALIDATOR_QUERY_PATH.to_string(), request_bytes)?;

    let response = QueryValidatorResponse::decode(response_bytes.as_slice())
        .map_err(|e| StdError::parse_err("QueryValidatorResponse", e))?;

    let validator = response
        .validator
        .ok_or_else(|| StdError::generic_err("chain returned no validator"))?;

    let status = BondStatus::try_from(validator.status)
        .unwrap_or(BondStatus::Unspecified)
        .as_str_name()
        .to_string();

    let moniker = validator
        .description
        .map(|d| d.moniker)
        .unwrap_or_default();

    let commission_rate = validator
        .commission
        .and_then(|c| c.commission_rates)
        .map(|r| r.rate)
        .unwrap_or_default();

    Ok(ValidatorResponse {
        operator_address: validator.operator_address,
        moniker,
        jailed: validator.jailed,
        status,
        tokens: validator.tokens,
        delegator_shares: validator.delegator_shares,
        commission_rate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmos_sdk_proto::cosmos::staking::v1beta1::{Commission, CommissionRates, Description};
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
                QueryRequest::Grpc(GrpcQuery { path, .. }) if path == VALIDATOR_QUERY_PATH => {
                    let response = mock_grpc_validator_response();
                    SystemResult::Ok(ContractResult::Ok(Binary::new(response.encode_to_vec())))
                }
                _ => SystemResult::Err(SystemError::UnsupportedRequest {
                    kind: "only the validator gRPC query is mocked".to_string(),
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

    fn mock_grpc_validator_response() -> QueryValidatorResponse {
        QueryValidatorResponse {
            validator: Some(cosmos_sdk_proto::cosmos::staking::v1beta1::Validator {
                operator_address: "cosmosvaloper1abc".to_string(),
                consensus_pubkey: None,
                jailed: false,
                status: BondStatus::Bonded as i32,
                tokens: "1000000".to_string(),
                delegator_shares: "1000000.000000000000000000".to_string(),
                description: Some(Description {
                    moniker: "Test Validator".to_string(),
                    identity: String::new(),
                    website: String::new(),
                    security_contact: String::new(),
                    details: String::new(),
                }),
                unbonding_height: 0,
                unbonding_time: None,
                commission: Some(Commission {
                    commission_rates: Some(CommissionRates {
                        rate: "0.050000000000000000".to_string(),
                        max_rate: "0.200000000000000000".to_string(),
                        max_change_rate: "0.010000000000000000".to_string(),
                    }),
                    update_time: None,
                }),
                min_self_delegation: "1".to_string(),
                unbonding_on_hold_ref_count: 0,
                unbonding_ids: vec![],
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
    fn query_validator_decodes_grpc_response() {
        let deps = mock_grpc_deps();

        let res = query_validator(deps.as_ref(), "cosmosvaloper1abc".to_string()).unwrap();
        assert_eq!(res.operator_address, "cosmosvaloper1abc");
        assert_eq!(res.moniker, "Test Validator");
        assert!(!res.jailed);
        assert_eq!(res.status, "BOND_STATUS_BONDED");
        assert_eq!(res.tokens, "1000000");
        assert_eq!(res.commission_rate, "0.050000000000000000");
    }
}
