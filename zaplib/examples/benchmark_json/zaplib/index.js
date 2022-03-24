const init = async () => {
    await zaplib.initialize({ wasmModule: '/target/wasm32-unknown-unknown/release/benchmark_json_zaplib.wasm' });
    await zaplib.callRustAsync(""); 
}

init();
