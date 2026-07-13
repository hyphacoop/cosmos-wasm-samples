use cosmwasm_std::{
    entry_point, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, VoteOption as CosmosVoteOption, WeightedVoteOption as CosmosWeightedVoteOption,
    GovMsg, StakingMsg,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InfoResponse, InstantiateMsg, QueryMsg, VoteOption};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:delegate-vote";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Delegate { validator, amount } => {
            let msg = CosmosMsg::Staking(StakingMsg::Delegate { validator, amount });
            Ok(Response::new()
                .add_message(msg)
                .add_attribute("method", "delegate"))
        }
        ExecuteMsg::Vote {
            proposal_id,
            option,
        } => {
            let cosmos_option = match option {
                VoteOption::Yes => CosmosVoteOption::Yes,
                VoteOption::No => CosmosVoteOption::No,
                VoteOption::Abstain => CosmosVoteOption::Abstain,
                VoteOption::NoWithVeto => CosmosVoteOption::NoWithVeto,
            };
            let msg = CosmosMsg::Gov(GovMsg::Vote {
                proposal_id,
                option: cosmos_option,
            });
            Ok(Response::new()
                .add_message(msg)
                .add_attribute("method", "vote")
                .add_attribute("proposal_id", proposal_id.to_string()))
        }
        ExecuteMsg::WeightedVote {
            proposal_id,
            options,
        } => {
            let cosmos_options: Vec<CosmosWeightedVoteOption> = options
                .into_iter()
                .map(|opt| {
                    let cosmos_option = match opt.option {
                        VoteOption::Yes => CosmosVoteOption::Yes,
                        VoteOption::No => CosmosVoteOption::No,
                        VoteOption::Abstain => CosmosVoteOption::Abstain,
                        VoteOption::NoWithVeto => CosmosVoteOption::NoWithVeto,
                    };
                    CosmosWeightedVoteOption {
                        option: cosmos_option,
                        weight: opt.weight,
                    }
                })
                .collect();
            let msg = CosmosMsg::Gov(GovMsg::VoteWeighted {
                proposal_id,
                options: cosmos_options,
            });
            Ok(Response::new()
                .add_message(msg)
                .add_attribute("method", "weighted_vote")
                .add_attribute("proposal_id", proposal_id.to_string()))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetInfo {} => to_json_binary(&query_info()?),
    }
}

fn query_info() -> StdResult<InfoResponse> {
    Ok(InfoResponse {
        info: "Delegate and Vote Contract".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, message_info};
    use cosmwasm_std::{coins, Addr};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "earth"));
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn delegate_message() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "earth"));
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let delegate_msg = ExecuteMsg::Delegate {
            validator: "cosmosvaloper1...".to_string(),
            amount: cosmwasm_std::Coin {
                denom: "uatom".to_string(),
                amount: 1000u128.into(),
            },
        };
        let res = execute(deps.as_mut(), mock_env(), info, delegate_msg).unwrap();
        assert_eq!(1, res.messages.len());
    }

    #[test]
    fn vote_message() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "earth"));
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let vote_msg = ExecuteMsg::Vote {
            proposal_id: 1,
            option: VoteOption::Yes,
        };
        let res = execute(deps.as_mut(), mock_env(), info, vote_msg).unwrap();
        assert_eq!(1, res.messages.len());
    }

    #[test]
    fn weighted_vote_message() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "earth"));
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let weighted_vote_msg = ExecuteMsg::WeightedVote {
            proposal_id: 1,
            options: vec![
                crate::msg::WeightedVoteOption {
                    option: VoteOption::Yes,
                    weight: cosmwasm_std::Decimal::percent(60),
                },
                crate::msg::WeightedVoteOption {
                    option: VoteOption::Abstain,
                    weight: cosmwasm_std::Decimal::percent(40),
                },
            ],
        };
        let res = execute(deps.as_mut(), mock_env(), info, weighted_vote_msg).unwrap();
        assert_eq!(1, res.messages.len());
    }
}
