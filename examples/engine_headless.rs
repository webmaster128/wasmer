//! Defining an engine in Wasmer is one of the fundamental steps.
//!
//! This example illustrates a neat feature of engines: their ability
//! to run in a headless mode. At the time of writing, all engines
//! have a headless mode, but it's not a requirement of the `Engine`
//! trait (defined in the `wasmer_engine` crate).
//!
//! What problem does it solve, and what does it mean?
//!
//! Once a Wasm module is compiled into executable code and stored
//! somewhere (e.g. in memory with the Universal engine), the module
//! can be instantiated and executed. But imagine for a second the
//! following scenario:
//!
//!   * Modules are compiled ahead of time, to be instantiated later
//!     on.
//!   * Modules are cross-compiled on a machine ahead of time
//!     to be run on another machine later one.
//!
//! In both scenarios, the environment where the compiled Wasm module
//! will be executed can be very constrained. For such particular
//! contexts, Wasmer can be compiled _without_ the compilers, so that
//! the `wasmer` binary is as small as possible. Indeed, there is no
//! need for a compiler since the Wasm module is already compiled. All
//! we need is an engine that _only_ drives the instantiation and
//! execution of the Wasm module.
//!
//! And that, that's a headless engine.
//!
//! To achieve such a scenario, a Wasm module must be compiled, then
//! serialized —for example into a file—, then later, potentially on
//! another machine, deserialized. The next steps are classical: The
//! Wasm module is instantiated and executed.
//!
//! This example uses a `compiler` because it illustrates the entire
//! workflow, but keep in mind the compiler isn't required after the
//! compilation step.
//!
//! You can run the example directly by executing in Wasmer root:
//!
//! ```shell
//! cargo run --example engine-headless --release --features "cranelift"
//! ```
//!
//! Or
//!
//! ```sh
//! cargo build --example engine-headless --release --features "cranelift"
//! codesign -s - -v -f --entitlements debug.plist ./target/release/examples/engine-headless
//! xcrun xctrace record --template 'Allocations' --launch ./target/release/examples/engine-headless
//! open -a Instruments Launch_engine-headless_*.trace
//! ```
//!
//! Ready?

use tempfile::NamedTempFile;
use wasmer::{imports, Engine, Function, Instance, Module, NativeEngineExt, Store};
use wasmer_compiler_cranelift::Cranelift;

const WASM: &[u8] = include_bytes!("nois_drand.wasm");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sleep a bit to allow starting profiling tools
    println!("Sleeping...");
    std::thread::sleep(std::time::Duration::from_secs(5));
    println!("Done sleeping.");

    // First step, let's compile the Wasm module and serialize it.
    // Note: we need a compiler here.
    let serialized_module_file = {
        // Define a compiler configuration.
        //
        // In this situation, the compiler is
        // `wasmer_compiler_cranelift`. The compiler is responsible to
        // compile the Wasm module into executable code.
        let compiler = Cranelift::default();

        // Create a store, that holds the engine.
        let store = Store::new(compiler);

        println!("Compiling module...");
        // Let's compile the Wasm module.
        let module = Module::new(&store, WASM)?;

        println!("Serializing module...");
        // Here we go. Let's serialize the compiled Wasm module in a
        // file.
        let serialized_module_file = NamedTempFile::new()?;
        module.serialize_to_file(&serialized_module_file)?;

        serialized_module_file
    };

    // We create a headless Universal engine.
    let runtime_engine = Engine::headless();

    // Second step, deserialize the compiled Wasm module, and execute
    // it, for example with Wasmer without a compiler.
    for i in 1..=10_000 {
        println!("Creating headless Universal engine {i}...");
        let mut store = Store::new(runtime_engine.clone());

        // println!("Deserializing module...");
        // Here we go.
        //
        // Deserialize the compiled Wasm module. This code is unsafe
        // because Wasmer can't assert the bytes are valid (see the
        // `wasmer::Module::deserialize`'s documentation to learn
        // more).
        let module =
            unsafe { Module::deserialize_from_file(&store, serialized_module_file.path()) }?;

        // Congrats, the Wasm module has been deserialized! Now let's
        // execute it for the sake of having a complete example.

        // Create an import object. Since our Wasm module didn't declare
        // any imports, it's an empty object.
        let import_object = imports! {
            "env" => {
                "db_read" => Function::new_typed(&mut store, |_a: u32| -> u32 { 0 }),
                "db_write" => Function::new_typed(&mut store, |_a: u32, _b: u32| {}),
                "db_remove" => Function::new_typed(&mut store, |_a: u32| {}),
                "db_scan" => Function::new_typed(&mut store, |_a: u32, _b: u32, _c: i32| -> u32 { 0 }),
                "db_next" => Function::new_typed(&mut store, |_a: u32| -> u32 { 0 }),
                "db_next_key" => Function::new_typed(&mut store, |_a: u32| -> u32 { 0 }),
                "db_next_value" => Function::new_typed(&mut store, |_a: u32| -> u32 { 0 }),
                "query_chain" => Function::new_typed(&mut store, |_a: u32| -> u32 { 0 }),
                "addr_validate" => Function::new_typed(&mut store, |_a: u32| -> u32 { 0 }),
                "addr_canonicalize" => Function::new_typed(&mut store, |_a: u32, _b: u32| -> u32 { 0 }),
                "addr_humanize" => Function::new_typed(&mut store, |_a: u32, _b: u32| -> u32 { 0 }),
                "secp256k1_verify" => Function::new_typed(&mut store, |_a: u32, _b: u32, _c: u32| -> u32 { 0 }),
                "secp256k1_recover_pubkey" => Function::new_typed(&mut store, |_a: u32, _b: u32, _c: u32| -> u64 { 0 }),
                "ed25519_verify" => Function::new_typed(&mut store, |_a: u32, _b: u32, _c: u32| -> u32 { 0 }),
                "ed25519_batch_verify" => Function::new_typed(&mut store, |_a: u32, _b: u32, _c: u32| -> u32 { 0 }),
                "debug" => Function::new_typed(&mut store, |_a: u32| {}),
                "abort" => Function::new_typed(&mut store, |_a: u32| {}),
            },
        };

        // println!("Instantiating module...");
        // Let's instantiate the Wasm module.
        let _instance = Instance::new(&mut store, &module, &import_object)?;

        // drop module, artifact should disappear
    }

    std::mem::drop(runtime_engine);

    // Show memory is good now
    println!("Sleeping...");
    std::thread::sleep(std::time::Duration::from_secs(5));
    println!("Done sleeping.");

    Ok(())
}

#[test]
#[cfg(not(any(windows, target_arch = "aarch64", target_env = "musl")))]
fn test_engine_headless() -> Result<(), Box<dyn std::error::Error>> {
    main()
}
