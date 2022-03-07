// This is the universal Zaplib Runtime which will work on both CEF and WebAssembly environments,
// doing runtime detection of which modules to load. No other file besides this one should conditionally
// branch based on environments, such that cef/wasm runtimes can work without including unnecessary code.

import * as wasm from "wasm_runtime";
import * as cef from "cef_runtime";
import { jsRuntime } from "type_of_runtime";
import { isZapBuffer } from "zap_buffer";
import { CreateBufferWorkerSync } from "types";

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

  //  where __zaplibCreateMutableBuffer returns a Vec<u8> of the given byteLength (the buffer should be initialized empty; we're going to copy in user data next on the JS side)
  const [buffer] = callRustSync("__zaplibCreateMutableBuffer", [
    bufferLen.toString(),
  ]);
  // in JS, copy the given data into the buffer.
  // TODO - how do I do this?

  return buffer as typeof data;
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
