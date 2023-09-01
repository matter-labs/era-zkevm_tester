pub mod compiler_tests;
pub mod debug_tracer;
pub mod events;
pub mod hashmap_based_memory;
pub mod simple_witness_tracer;
mod vm2_runner;

use crate::trace::VmTrace;
use crate::Address;

pub use tracing as __tracing;
pub use tracing::{debug, info, trace};

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::runners::__tracing::warn!(
            file=file!(),
            line=line!(),
            column=column!(),
            $($arg)*
        )
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::runners::__tracing::info!(
            file=file!(),
            line=line!(),
            column=column!(),
            $($arg)*
        )
    };
}

pub fn output_execution_trace(trace: VmTrace, _entry_address: Address, test_name: String) {
    let mut file_name = format!("{}_dump.json", test_name);
    if let Ok(env_name) = std::env::var("ZKEVM_TRACE_OUTPUT_FILE") {
        file_name = env_name;
    }
    // let file_name = std::env::var("ZKEVM_TRACE_OUTPUT_FILE").unwrap_or(format!(
    //     "zkEVM-trace-{}-{}.json",
    //     entry_address,
    //     std::time::SystemTime::now()
    //         .duration_since(std::time::UNIX_EPOCH)
    //         .unwrap()
    //         .as_millis()
    // ));

    let steps = trace.steps.len();

    std::fs::File::create(file_name.clone()).map_or_else(
        |e| warn!("Unable to create file {}: {}", file_name, e),
        |file| match serde_json::to_writer(&file, &trace) {
            Err(err) => warn!("Unable to write trace to file {}: {}", file_name, err),
            Ok(_) => info!("Wrote trace ({} steps) to file {}", steps, file_name),
        },
    )
}
