// TODO: setup lints
#![allow(dead_code)]

use auraescript_deno::*;
use deno_core::resolve_path;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // only supports a single script for now
    if args.len() != 2 {
        println!("Usage: auraescript <path_to_module>");
        std::process::exit(1);
    }

    let mut js_runtime = init();

    let main_module = resolve_path(&args[1].clone())?;

    let future = async move {
        let mod_id = js_runtime.load_main_module(&main_module, None).await?;
        let result = js_runtime.mod_evaluate(mod_id);
        js_runtime.run_event_loop(false).await?;
        result.await?
    };

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to initialize tokio runtime")
        .block_on(future)
}
