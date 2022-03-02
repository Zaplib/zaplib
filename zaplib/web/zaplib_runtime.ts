// This is the universal Zaplib Runtime which will work on both CEF and WebAssembly environments,
// doing runtime detection of which modules to load. No other file besides this one should conditionally
// branch based on environments, such that cef/wasm runtimes can work without including unnecessary code.

import * as wasm from "wasm_runtime";
import * as cef from "cef_runtime";
import { jsRuntime } from "type_of_runtime";
import { isZapBuffer } from "zap_buffer";
import "./zaplib.css";

const {
  initialize,
  isInitialized,
  newWorkerPort,
  registerCallJsCallbacks,
  unregisterCallJsCallbacks,
  callRustAsync,
  serializeZapArrayForPostMessage,
  deserializeZapArrayFromPostMessage,
  callRustSync,
  createMutableBuffer,
  createReadOnlyBuffer,
} = jsRuntime === "cef" ? cef : wasm;

export {
  initialize,
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
