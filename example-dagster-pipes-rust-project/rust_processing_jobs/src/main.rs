use dagster_pipes_rust::open_dagster_pipes;
use serde_json::json;

fn main() {
    let mut context = open_dagster_pipes();
    // See supported metadata types here:
    // https://github.com/dagster-io/dagster/blob/master/python_modules/dagster/dagster/_core/pipes/context.py#L133
    let metadata = json!({"row_count": {"raw_value": 100, "type": "int"}});
    context.report_asset_materialization("example_rust_subprocess_asset", metadata);
}