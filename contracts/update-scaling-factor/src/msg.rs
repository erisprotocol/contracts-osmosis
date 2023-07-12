use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub pool_id: u64,
    pub scale_first: bool,
    pub hub: String,
    pub owner: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateScalingFactor {},
    UpdateConfig {
        pool_id: Option<u64>,
        hub: Option<String>,
        scale_first: Option<bool>,
    },
    UpdateOwnership(cw_ownable::Action),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},

    #[returns(cw_ownable::Ownership<String> )]
    Ownership {},
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct Config {
    pub pool_id: u64,
    pub hub: Addr,
    pub scale_first: bool,
}
