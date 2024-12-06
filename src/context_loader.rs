use serde_json::{from_str, from_value, Map, Value};

use crate::PipesContextData;

pub trait PipesContextLoader {
    fn load_context(&self, params: Map<String, Value>) -> PipesContextData;
}

/// Context loader that loads context data from either a file or directly from the provided params.
///
/// The location of the context data is configured by the params received by the loader. If the params
/// include a key `path`, then the context data will be loaded from a file at the specified path. If
/// the params instead include a key `data`, then the corresponding value should be a dict
/// representing the context data.
/// Translation of https://github.com/dagster-io/dagster/blob/258d9ca0db7fcc16d167e55fee35b3cf3f125b2e/python_modules/dagster-pipes/dagster_pipes/__init__.py#L604-L630
#[derive(Debug, Default)]
pub struct PipesDefaultContextLoader;

impl PipesDefaultContextLoader {
    pub fn new() -> Self {
        Self
    }
}

impl PipesContextLoader for PipesDefaultContextLoader {
    fn load_context(&self, params: Map<String, Value>) -> PipesContextData {
        // TODO: Have this function return a `Result<PipesContextData>` instead
        const FILE_PATH_KEY: &str = "path";
        const DIRECT_KEY: &str = "data";

        match (params.get(FILE_PATH_KEY), params.get(DIRECT_KEY)) {
            // `_` in second-half of tuple to account for the case where both keys are specified
            (Some(Value::String(path)), _) => {
                from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
            }
            (None, Some(Value::Object(map))) => from_value(Value::Object(map.clone())).unwrap(),
            _ => {
                panic!(
                    "Invalid params, expected key \"{}\" or \"{}\", received {:?}",
                    FILE_PATH_KEY, DIRECT_KEY, params
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io::Write;

    use serde_json::json;
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_load_context_from_data_dict() {
        let default_context_loader = PipesDefaultContextLoader::new();
        let params = json!({"data": {"asset_keys": ["asset1", "asset2"], "run_id": "0123456", "extras": {"key": "value"}}});
        let params = params.as_object().expect("Invalid raw JSON provided");
        assert_eq!(
            default_context_loader.load_context(params.clone()),
            PipesContextData {
                asset_keys: Some(vec!["asset1".to_string(), "asset2".to_string()]),
                run_id: "0123456".to_string(),
                extras: HashMap::from([("key".to_string(), Value::String("value".to_string()))])
            }
        );
    }

    #[test]
    fn test_load_context_from_file_path() {
        // TODO: This is an integration test (interacting with the filesystem)
        //       Mock the implementation instead.
        let default_context_loader = PipesDefaultContextLoader::new();
        let mut file = NamedTempFile::new().expect("Failed to create tempfile for testing");
        file.write_all(r#"{"asset_keys": ["asset1", "asset2"], "run_id": "0123456", "extras": {"key": "value"}}"#.as_bytes()).expect("Failed to write data into tempfile");

        let params = json!({"path": file.path()});
        let params = params.as_object().unwrap();
        assert_eq!(
            default_context_loader.load_context(params.clone()),
            PipesContextData {
                asset_keys: Some(vec!["asset1".to_string(), "asset2".to_string()]),
                run_id: "0123456".to_string(),
                extras: HashMap::from([("key".to_string(), Value::String("value".to_string()))])
            }
        );
    }

    /// Mimics behaviour on Python side
    #[test]
    fn test_load_context_prioritizes_file_path_when_both_are_present() {
        let default_context_loader = PipesDefaultContextLoader::new();
        let mut file = NamedTempFile::new().expect("Failed to create tempfile for testing");
        file.write_all(r#"{"asset_keys": ["asset_from_path"], "run_id": "id_from_path", "extras": {"key_from_path": "value_from_path"}}"#.as_bytes()).expect("Failed to write data into tempfile");

        let params = json!({"data": {"asset_keys": ["asset_from_data"], "run_id": "id_from_data", "extras": {"key_from_data": "value_from_data"}}, "path": file.path()});
        let params = params.as_object().unwrap();
        assert_eq!(
            default_context_loader.load_context(params.clone()),
            PipesContextData {
                asset_keys: Some(vec!["asset_from_path".to_string()]),
                run_id: "id_from_path".to_string(),
                extras: HashMap::from([(
                    "key_from_path".to_string(),
                    Value::String("value_from_path".to_string())
                )])
            }
        );
    }
}
