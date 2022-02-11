zaplib.initialize({ wasmModule: 'target/wasm32-unknown-unknown/debug/tutorial_js_rust_bridge.wasm', defaultStyles: true }).then(async () => {
    const values = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    const results = await zaplib.callRust('sum', [values]);
    const sumArray = results[0]
    const sum = sumArray[0];
    document.getElementById('root').textContent = sum;
});
