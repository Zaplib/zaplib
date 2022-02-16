/// <reference lib="WebWorker" />

import * as zaplib from "../zaplib_worker_runtime";
import { expect, expectThrow, expectThrowAsync } from "./test_helpers";
import { Rpc } from "../common";
import { TestSuiteWorkerSpec } from "./test_suite";
import { Worker } from "../rpc_types";
import { inWorker } from "../type_of_runtime";

const rpc = new Rpc<Worker<TestSuiteWorkerSpec>>(self);

const tests = {
  testCallRustFromWorker: async function () {
    const buffer = new SharedArrayBuffer(8);
    new Uint8Array(buffer).set([1, 2, 3, 4, 5, 6, 7, 8]);
    const uint8Part = new Uint8Array(buffer, 2, 4);
    const [result] = await zaplib.callRust("array_multiply_u8", [
      JSON.stringify(10),
      uint8Part,
    ]);
    expect(result.length, 4);
    expect(result[0], 30);
    expect(result[1], 40);
    expect(result[2], 50);
    expect(result[3], 60);
  },
  testCallRustNoReturnFromWorker: async function () {
    const buffer = new SharedArrayBuffer(8);
    new Uint8Array(buffer).set([1, 2, 3, 4, 5, 6, 7, 8]);
    const uint8Part = new Uint8Array(buffer, 2, 4);
    const result = await zaplib.callRust("call_rust_no_return", [
      JSON.stringify(10),
      uint8Part,
    ]);
    expect(result.length, 0);
  },
  testCallRustInSameThreadSyncWithSignal: function () {
    const result = zaplib.callRustInSameThreadSync("send_signal");
    expect(result.length, 0);
  },
  testCallRustFloat32ArrayFromWorker: async () => {
    // Using a normal array
    const input = new Float32Array([0.1, 0.9, 0.3]);
    const result = (
      await zaplib.callRust("array_multiply_f32", [JSON.stringify(10), input])
    )[0] as Float32Array;
    expect(result.length, 3);
    expect(result[0], 1);
    expect(result[1], 9);
    expect(result[2], 3);

    // Using a ZapArray
    const input2 = zaplib.createMutableBuffer(
      new Float32Array([0.1, 0.9, 0.3])
    );
    const result2 = (
      await zaplib.callRust("array_multiply_f32", [JSON.stringify(10), input2])
    )[0] as Float32Array;
    expect(result2.length, 3);
    expect(result2[0], 1);
    expect(result2[1], 9);
    expect(result2[2], 3);

    // Using a readonly ZapArray
    const input3 = zaplib.createReadOnlyBuffer(
      new Float32Array([0.1, 0.9, 0.3])
    );

    const result3 = (
      await zaplib.callRust("array_multiply_f32_readonly", [
        JSON.stringify(10),
        input3,
      ])
    )[0] as Float32Array;
    expect(result3.length, 3);
    expect(result3[0], 1);
    expect(result3[1], 9);
    expect(result3[2], 3);
  },
  testCallRustInSameThreadSyncFloat32ArrayFromWorker: async () => {
    // Using a normal array
    const input = new Float32Array([0.1, 0.9, 0.3]);
    const result = zaplib.callRustInSameThreadSync("array_multiply_f32", [
      JSON.stringify(10),
      input,
    ])[0] as Float32Array;
    expect(result.length, 3);
    expect(result[0], 1);
    expect(result[1], 9);
    expect(result[2], 3);

    // Using a ZapArray
    const input2 = zaplib.createMutableBuffer(
      new Float32Array([0.1, 0.9, 0.3])
    );
    const result2 = zaplib.callRustInSameThreadSync("array_multiply_f32", [
      JSON.stringify(10),
      input2,
    ])[0] as Float32Array;
    expect(result2.length, 3);
    expect(result2[0], 1);
    expect(result2[1], 9);
    expect(result2[2], 3);

    // Using a readonly ZapArray
    const input3 = zaplib.createReadOnlyBuffer(
      new Float32Array([0.1, 0.9, 0.3])
    );

    const result3 = zaplib.callRustInSameThreadSync(
      "array_multiply_f32_readonly",
      [JSON.stringify(10), input3]
    )[0] as Float32Array;
    expect(result3.length, 3);
    expect(result3[0], 1);
    expect(result3[1], 9);
    expect(result3[2], 3);
  },
  testInWorker: () => {
    expect(inWorker, true);
  },
  testErrorAfterPanic: async () => {
    // all calls to Rust should fail after this
    const funcs = [
      () => zaplib.callRustInSameThreadSync("call_rust_no_return"),
      () => zaplib.createMutableBuffer(new Uint8Array()),
      () => zaplib.createReadOnlyBuffer(new Uint8Array()),
    ];
    for (const f of funcs) {
      expectThrow(f, "Zaplib WebAssembly instance crashed");
    }
    await expectThrowAsync(
      () => zaplib.callRust("call_rust_no_return"),
      "Zaplib WebAssembly instance crashed"
    );
  },
};
export type TestSuiteTests = keyof typeof tests;

rpc.receive("initWasm", (port) => zaplib.initializeWorker(port));

rpc.receive("runTest", async (testName) => tests[testName]());

rpc.receive("sendWorker", function (array) {
  const data = zaplib.deserializeZapArrayFromPostMessage(array);
  console.log("got data", data);
});

rpc.receive("testSendZapArrayToMainThread", function () {
  const buffer = new SharedArrayBuffer(8);
  new Uint8Array(buffer).set([1, 2, 3, 4, 5, 6, 7, 8]);
  const uint8Part = new Uint8Array(buffer, 2, 4);
  const zapArray = zaplib.callRustInSameThreadSync("array_multiply_u8", [
    JSON.stringify(10),
    uint8Part,
  ])[0] as Uint8Array;

  return {
    array: zaplib.serializeZapArrayForPostMessage(zapArray),
    subarray: zaplib.serializeZapArrayForPostMessage(zapArray.subarray(1, 3)),
  };
});
rpc.receive("testCallRustInSameThreadSyncWithZapbuffer", function () {
  const result = zaplib.createMutableBuffer(
    new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8])
  );
  const [result2] = zaplib.callRustInSameThreadSync("array_multiply_u8", [
    JSON.stringify(10),
    result,
  ]);

  // Needed for type refinement.
  if (typeof result2 === "string") {
    throw new Error("didn't expect result2 to be a string");
  }

  return zaplib.serializeZapArrayForPostMessage(result2);
});
