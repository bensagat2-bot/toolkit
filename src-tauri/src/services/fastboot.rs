use super::adb;

pub async fn run_command(args: Vec<String>) -> String {
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    adb::run_fastboot(&arg_refs).await
}
