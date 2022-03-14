// This is the universal Zaplib Runtime which will work on both CEF and WebAssembly environments,
// doing runtime detection of which modules to load. No other file besides this one should conditionally
// branch based on environments, such that cef/wasm runtimes can work without including unnecessary code.

import * as wasm from "wasm_runtime";
import * as cef from "cef_runtime";
import { jsRuntime } from "type_of_runtime";
import { isZapBuffer } from "zap_buffer";
import { CreateBufferWorkerSync } from "types";
// import { copyArrayToRustBuffer } from "common";

const {
  initialize,
  close,
  isInitialized,
  newWorkerPort,
  registerCallJsCallbacks,
  unregisterCallJsCallbacks,
  callRustAsync,
  serializeZapArrayForPostMessage,
  deserializeZapArrayFromPostMessage,
  callRustSync,
  createReadOnlyBuffer,
} = jsRuntime === "cef" ? cef : wasm;

const createMutableBuffer: CreateBufferWorkerSync = (data) => {
  const bufferLen = data.byteLength;

  //  __zaplibCreateMutableBuffer returns an empty Vec<u8> of the given byteLength
  const [buffer] = callRustSync("__zaplibCreateMutableBuffer", [
    bufferLen.toString(), // TODO (Steve) - allow numbers as ZapParams
  ]);
  // JP suggested this might be relevant, but this call doesn't seem to do anything:
  // copyArrayToRustBuffer(data, buffer as ArrayBuffer, 0);

  // this does set the data:
  (buffer as typeof data).set(data, 0);

  // this works as long as the type of the input data is UInt8Array
  return buffer as typeof data;

  // So I tried creating a typed array the type of the input data
  // It seems to work ok, except it fails on this line of the test helper:
  // https://github.com/Zaplib/zaplib/blob/74c0058e1a13ce2f10e320c262bf1dbb8f2371c8/zaplib/web/test_suite/test_helpers.ts#L134

  // @ts-ignore: constructor is getting typed as Function instead of a constructor
  // return new data.constructor(buffer, 0, bufferLen);
};

// export const createReadOnlyBufferImpl: CreateBuffer = async (data) => {
//   // That solves for the mutable buffer case. In the read-only buffer case, you do the above to create a mutable buffer, and then send it back to Rust via to move the data into an Arc<Vec<…>> via __zaplibMakeBufferReadOnly, then return that, and now you have your read-only buffer.
// }

export {
  initialize,
  close,
  isInitialized,
  newWorkerPort,
  registerCallJsCallbacks,
  unregisterCallJsCallbacks,
  callRustAsync,
  serializeZapArrayForPostMessage,
  deserializeZapArrayFromPostMessage,
  callRustSync,
  jsRuntime,
  createMutableBuffer,
  createReadOnlyBuffer,
  isZapBuffer,
};
