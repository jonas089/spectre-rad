use crate::types::SyncStepArgs;
use std::{env, fs};

pub fn load_circuit_args(path: &str) -> SyncStepArgs {
    return serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
}

pub fn load_circuit_args_env() -> SyncStepArgs {
    let path = env::var("SYNC_STEP_TEST_PATH").unwrap_or("../data/sync_step_512.json".to_string());
    return serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
}

#[cfg(test)]
mod tests {
    use super::load_circuit_args_env;
    #[test]
    fn test_load_circuit_args() {
        let _args = load_circuit_args_env();
    }
}
