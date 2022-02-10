// Imports self into NodeJS context
require("node-self");

// Needed for async tests in jest
require("regenerator-runtime/runtime");

// Importing the worker polyfill
// @ts-ignore
// eslint-disable-next-line @typescript-eslint/no-var-requires
const Worker = require("../vendor/web-worker/node");
self.Worker = Worker;

// @ts-ignore
// eslint-disable-next-line @typescript-eslint/no-var-requires
const { sendToDummyWorker } = require("../dist/test_jest");

test("calls dummy worker", async () => {
  const result = await sendToDummyWorker("foo");
  expect(result).toBe("dummy:foo");
});
