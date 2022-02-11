/* eslint-disable */

// For async tests in jest
require("regenerator-runtime/runtime");

require("../dist/zaplib_nodejs_polyfill");

const fs = require("fs");

// @ts-ignore
// eslint-disable-next-line @typescript-eslint/no-var-requires
const { sendToDummyWorker } = require("../dist/test_jest");

// @ts-ignore
// eslint-disable-next-line @typescript-eslint/no-var-requires
const zaplib = require("../dist/zaplib_runtime");

test("calls dummy worker", async () => {
  const result = await sendToDummyWorker("foo");
  expect(result).toBe("dummy:foo");
});

test("initializes zaplib and calls rust", async () => {
  const wasmPath = "../../target/wasm32-unknown-unknown/debug/test_suite.wasm";
  const wasmModule = WebAssembly.compile(fs.readFileSync(wasmPath));
  await zaplib.initialize({ wasmModule: wasmModule });
  const buffer = new SharedArrayBuffer(8);
  const data = new Uint8Array(buffer);
  data.set([1, 2, 3, 4, 5, 6, 7, 8]);
  const [result] = await zaplib.callRust("total_sum", [data]);
  expect(result).toBe("36");
});
