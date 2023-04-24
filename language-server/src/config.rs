use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub always_used_rule_names: Vec<String>,
    pub check_for_updates: bool,
    pub enable_performance_logging: bool,
}
