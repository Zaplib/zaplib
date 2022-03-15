// This is the universal Zaplib Runtime which will work on both CEF and WebAssembly environments,
// doing runtime detection of which modules to load. No other file besides this one should conditionally
// branch based on environments, such that cef/wasm runtimes can work without including unnecessary code.

import * as wasm from "wasm_runtime";
import * as cef from "cef_runtime";
import { jsRuntime } from "type_of_runtime";
import { isZapBuffer } from "zap_buffer";
import { CreateBuffer } from "types";
import { getZapParamType } from "common";

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
} = jsRuntime === "cef" ? cef : wasm;

const createMutableBuffer: CreateBuffer = (data) => {
  const [buffer] = callRustSync<[typeof data]>("__zaplibCreateMutableBuffer", [
    getZapParamType(data, false).toString(),
    data.length.toString(),
  ]);
  buffer.set(data, 0);

  return buffer;
};

const createReadOnlyBuffer: CreateBuffer = (data) => {
  const buffer = createMutableBuffer(data);

  const [readOnlyBuffer] = callRustSync<[typeof data]>(
    "__zaplibMakeBufferReadOnly",
    [buffer]
  );

  return readOnlyBuffer;
};

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
