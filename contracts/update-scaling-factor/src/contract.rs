use std::ops::Div;

const CONTRACT_NAME: &str = "eris-update-scaling-factor";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

use crate::{
    error::{ContractError, ContractResult, CustomResult},
    msg::{Config, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::CONFIG,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, Fraction, MessageInfo, Response, StdResult, Storage,
    Uint128,
};
use cw2::set_contract_version;
use cw_ownable::{get_ownership, update_ownership};
use eris::hub::{QueryMsg as HubQueryMsg, StateResponse};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.owner))?;
    CONFIG.save(
        deps.storage,
        &Config {
            pool_id: msg.pool_id,
            hub: deps.api.addr_validate(&msg.hub)?,
            scale_first: msg.scale_first,
        },
    )?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ContractResult {
    match msg {
        ExecuteMsg::UpdateOwnership(action) => {
            update_ownership(deps, &env.block, &info.sender, action)?;
            Ok(Response::new().add_attribute("action", "update_ownership"))
        }
        ExecuteMsg::UpdateScalingFactor {} => update_scaling_factor(deps, env, info),
        ExecuteMsg::UpdateConfig { .. } => update_config(deps, env, info, msg),
    }
}

fn update_scaling_factor(deps: DepsMut, env: Env, _info: MessageInfo) -> ContractResult {
    let config = CONFIG.load(deps.storage)?;

    let state: StateResponse = deps
        .querier
        .query_wasm_smart(config.hub, &HubQueryMsg::State {})?;
    let exchange_rate = state.exchange_rate;

    let scaling_factors = if config.scale_first {
        get_factors(exchange_rate.numerator(), exchange_rate.denominator())?
    } else {
        get_factors(exchange_rate.denominator(), exchange_rate.numerator())?
    };

    let factors = scaling_factors
        .iter()
        .map(|a| a.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let msg = osmosis_std::types::osmosis::gamm::poolmodels::stableswap::v1beta1::MsgStableSwapAdjustScalingFactors {
        pool_id: config.pool_id,
        sender: env.contract.address.to_string(),
        scaling_factors: scaling_factors
    };

    Ok(Response::new()
        .add_attribute("action", "update_scaling_factor")
        .add_attribute("factors", factors)
        .add_message(msg))
}

fn get_factors(numerator: Uint128, denominator: Uint128) -> CustomResult<Vec<u64>> {
    // 10^9 -> 9 numbers after the comma
    let first = numerator.u128().div(1000000000);
    let second = denominator.u128().div(1000000000);

    Ok(vec![u64::try_from(first)?, u64::try_from(second)?])
}

fn update_config(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> ContractResult {
    match msg {
        ExecuteMsg::UpdateConfig {
            pool_id: pool,
            hub,
            scale_first,
        } => {
            cw_ownable::assert_owner(deps.storage, &info.sender)?;

            CONFIG.update(deps.storage, |mut config| -> StdResult<Config> {
                if let Some(pool) = pool {
                    config.pool_id = pool
                };
                if let Some(scale_first) = scale_first {
                    config.scale_first = scale_first
                };

                if let Some(hub) = hub {
                    config.hub = deps.api.addr_validate(hub.as_str())?;
                };

                Ok(config)
            })?;

            Ok(Response::new().add_attribute("action", "update_config"))
        }
        _ => Err(ContractError::NotSupported),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => to_binary(&get_ownership(deps.storage)?),
        QueryMsg::Config {} => to_binary(&get_config(deps.storage)?),
    }
}

fn get_config(store: &dyn Storage) -> StdResult<Config> {
    CONFIG.load(store)
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResult {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("new_contract_name", CONTRACT_NAME)
        .add_attribute("new_contract_version", CONTRACT_VERSION))
}

#[cfg(test)]
mod tests {
    use super::get_factors;
    use cosmwasm_std::{Decimal, Fraction};
    use std::str::FromStr;

    #[test]
    fn test_scaling_factors() {
        let exchange_rate = Decimal::from_str("1.19234").unwrap();
        let factors = get_factors(exchange_rate.numerator(), exchange_rate.denominator())
            .expect("should be set");
        assert_eq!(factors, vec![1192340000, 1000000000]);

        // cutoff after 9th position
        let exchange_rate = Decimal::from_str("1.192342342456").unwrap();
        let factors = get_factors(exchange_rate.numerator(), exchange_rate.denominator())
            .expect("should be set");
        assert_eq!(factors, vec![1192342342, 1000000000]);

        // Decimal::MAX too big
        let exchange_rate = Decimal::MAX;
        let factors = get_factors(exchange_rate.numerator(), exchange_rate.denominator())
            .expect_err("should be error");
        assert_eq!(
            factors.to_string(),
            "out of range integral type conversion attempted"
        );

        // max exchange_rate that is supported
        let exchange_rate = Decimal::from_str("18446744073.709551615").unwrap();
        let factors = get_factors(exchange_rate.numerator(), exchange_rate.denominator())
            .expect("should be set");
        assert_eq!(factors, vec![u64::MAX, 1000000000]);
    }
}
