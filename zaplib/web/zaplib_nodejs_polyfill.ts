// This is a convenient set of polyfills for running Zaplib in Node.js.
// The idea is that you only need to import these polyfills to get Zaplib to run in Node.js.

// Worker poylfil to use Webworkers
// @ts-ignore
import Worker from "vendor/web-worker/node";
globalThis.Worker = Worker;

// eslint-disable-next-line
const threads = require("worker_threads");
globalThis.MessageChannel = threads.MessageChannel;

// Webpack's worker-loader needs this.
// https://github.com/webpack-contrib/worker-loader/blob/a37f4b2caff11bb0bad5b54090a6de940504a3cb/src/runtime/inline.js#L5
// TODO(JP): worker-loader is deprecated, so see if we can get the equivalent
// behavior in Webpack's built-in worker-loader now; maybe then we don't need
// this anymore?
// @ts-ignore
globalThis.self ||= globalThis;
