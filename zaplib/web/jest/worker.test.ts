// Imports self into NodeJS context
import 'node-self'

// Needed for async tests in jest
import "regenerator-runtime/runtime";

// Importing the worker polyfill
// @ts-ignore
import Worker from "../vendor/web-worker/node";
self.Worker = Worker;

// @ts-ignore
import {sendToDummyWorker} from "../dist/test_jest";

test("calls dummy worker", async () => {
  let result = await sendToDummyWorker("foo");
  expect(result).toBe("dummy:foo");
});
