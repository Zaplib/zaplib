// We only define `cefCallRustAsync` if in CEF, so we can use this for environment detection.
// This should only be used at the top level `zaplib_runtime` file or in test, since we want to keep
// CEF and WASM code separate for bundle size.
export const jsRuntime = "cefCallRustAsync" in self ? "cef" : "wasm";

// Whether or not we're in a WebWorker.
// From https://stackoverflow.com/a/23619712
export const inWorker = typeof importScripts === "function";

// Only Node.JS has a process variable that is of `Class` `process`
// From https://github.com/iliakan/detect-node/blob/00381fd0fdbdefa625ac7b8230adfc1df11d49ad/index.js
export const inNodeJs =
  Object.prototype.toString.call(
    typeof process !== "undefined" ? process : 0
  ) === "[object process]";

// Injected using webpack.DefinePlugin.
declare const __GIT_SHA__: string;
export const gitSha = __GIT_SHA__;
