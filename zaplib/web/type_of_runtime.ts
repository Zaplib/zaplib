// We only define `cefCallRust` if in CEF, so we can use this for environment detection.
// This should only be used at the top level `zaplib_runtime` file or in test, since we want to keep
// CEF and WASM code separate for bundle size.
export const jsRuntime = "cefCallRust" in self ? "cef" : "wasm";

// Whether or not we're in a WebWorker.
// From https://stackoverflow.com/a/23619712
export const inWorker = typeof importScripts === "function";

// Injected using webpack.DefinePlugin.
declare const __GIT_SHA__: string;
export const gitSha = __GIT_SHA__;
