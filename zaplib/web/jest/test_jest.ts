// These are jest-specific tests that test subset of Zaplib functionality using NodeJS env.

// @ts-ignore
import DummyWorker from "worker-loader?inline=no-fallback!./dummy_worker";

export function sendToDummyWorker(s: string): Promise<any> {
  const worker = DummyWorker();
  const promise = new Promise((resolve) => {
    worker.addEventListener("message", (event: any) => {
      resolve(event.data);
    });
  });
  worker.postMessage(s);
  return promise;
}
