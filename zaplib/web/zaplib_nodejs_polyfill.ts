// This is a convenient set of polyfills for running Zaplib in Node.js.
// The idea is that you only need to import these polyfills to get Zaplib to run in Node.js.

// Worker poylfil to use Webworkers
// @ts-ignore
import Worker from "./vendor/web-worker/node";
self.Worker = Worker;
