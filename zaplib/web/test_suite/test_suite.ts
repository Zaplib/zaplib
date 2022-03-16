// @ts-ignore
import TestSuiteWorker from "worker-loader?inline=no-fallback!test_suite/test_suite_worker";

import { assertNotNull, Rpc } from "common";
import { TestSuiteTests } from "test_suite/test_suite_worker";
import { PostMessageTypedArray, ZapArray } from "types";
import { zapBufferTests } from "test_suite/zap_buffer_test";
import * as zaplib from "zaplib_runtime";
import {
  expect,
  expectDeallocationOrUnregister as _expectDeallocationOrUnregister,
  expectThrow,
  expectThrowAsync,
  setInTest,
} from "test_suite/test_helpers";
import { inWorker } from "type_of_runtime";

declare global {
  interface Window {
    // Exposed for zaplib_ci.
    runAllTests3x: () => Promise<void>;
  }
}

const expectDeallocationOrUnregister = (buffer: ZapArray) =>
  _expectDeallocationOrUnregister(zaplib.callRustAsync, buffer);

export type TestSuiteWorkerSpec = {
  send: {
    runTest: [TestSuiteTests, void];
    initWasm: [MessagePort, void];
    sendWorker: [PostMessageTypedArray, void];
    testSendZapArrayToMainThread: [
      void,
      {
        array: PostMessageTypedArray;
        subarray: PostMessageTypedArray;
      }
    ];
    testCallRustAsyncSyncWithZapbuffer: [void, PostMessageTypedArray];
  };
  receive: Record<string, never>;
};
const rpc = new Rpc<TestSuiteWorkerSpec>(new TestSuiteWorker());

const runWorkerTest = (testName: TestSuiteTests) => () =>
  rpc.send("runTest", testName);

const env = new URL(window.document.location.toString()).searchParams.has(
  "release"
)
  ? "release"
  : "debug";

let onPanicCalled = false;

expect(zaplib.isInitialized(), false);
zaplib
  .initialize({
    wasmModule: `target/wasm32-unknown-unknown/${env}/test_suite.wasm`,
    defaultStyles: true,
    onPanic: () => {
      onPanicCalled = true;
    },
  })
  .then(async () => {
    expect(zaplib.isInitialized(), true);

    // Initialize the worker by sending a "zap worker port" to it in the first message.
    if (zaplib.jsRuntime === "wasm") {
      const zapWorkerPort = zaplib.newWorkerPort();
      await rpc.send("initWasm", zapWorkerPort, [zapWorkerPort]);
    }

    zaplib.registerCallJsCallbacks({
      log(params) {
        console.log("log fn called", params[0]);
        const div = document.createElement("div");
        div.innerText = "log fn called: " + params[0];
        assertNotNull(document.getElementById("root")).append(div);
      },
      sendWorker(params) {
        const toSend = params[0] as Uint8Array;
        console.log("sending data", toSend);
        // Note: uncomment to see the error about sending typed arrays
        // worker.postMessage(buffers[0]);
        rpc.send("sendWorker", zaplib.serializeZapArrayForPostMessage(toSend));
      },
    });

    const runtimeSpecificTests =
      zaplib.jsRuntime === "wasm"
        ? {
            "Call rust from worker": runWorkerTest(
              "testCallRustAsyncFromWorker"
            ),
            "Call rust (no return) from worker": runWorkerTest(
              "testCallRustAsyncNoReturnFromWorker"
            ),
            "Call rust with Float32Array from worker": runWorkerTest(
              "testCallRustAsyncFloat32ArrayFromWorker"
            ),
            "Call rust in same thread sync with Float32Array from worker":
              runWorkerTest("testCallRustAsyncSyncFloat32ArrayFromWorker"),
            "Test that for a worker 'inWorker' returns true":
              runWorkerTest("testInWorker"),
            "Send zap array to main thread": async () => {
              const result = await rpc.send("testSendZapArrayToMainThread");

              const array = zaplib.deserializeZapArrayFromPostMessage(
                result.array
              );
              const subarray = zaplib.deserializeZapArrayFromPostMessage(
                result.subarray
              );

              expect(array.length, 4);
              expect(array[0], 30);
              expect(array[1], 40);
              expect(array[2], 50);
              expect(array[3], 60);

              expect(subarray.length, 2);
              expect(subarray[0], 40);
              expect(subarray[1], 50);
            },
            "Call Rust in same thread with zapbuffer from worker": async () => {
              const result = await rpc.send(
                "testCallRustAsyncSyncWithZapbuffer"
              );
              const array = zaplib.deserializeZapArrayFromPostMessage(result);
              expect(array.length, 8);
              expect(array[0], 10);
              expect(array[1], 20);
              expect(array[2], 30);
              expect(array[3], 40);
              expect(array[4], 50);
              expect(array[5], 60);
              expect(array[6], 70);
              expect(array[7], 80);
            },
            "Send signal from worker": runWorkerTest(
              "testCallRustAsyncSyncWithSignal"
            ),
          }
        : {
            // CEF
          };

    const tests = {
      "Call Rust": async () => {
        const buffer = new SharedArrayBuffer(8);
        new Uint8Array(buffer).set([1, 2, 3, 4, 5, 6, 7, 8]);
        const uint8Part = new Uint8Array(buffer, 2, 4);
        const [result] = await zaplib.callRustAsync("array_multiply_u8", [
          JSON.stringify(10),
          uint8Part,
        ]);
        expect(result.length, 4);
        expect(result[0], 30);
        expect(result[1], 40);
        expect(result[2], 50);
        expect(result[3], 60);
      },
      "Call Rust (no return)": async () => {
        const result = await zaplib.callRustAsync("call_rust_no_return");
        expect(result.length, 0);
      },
      "Call Rust (string return)": async () => {
        const buffer = new SharedArrayBuffer(8);
        const data = new Uint8Array(buffer);
        data.set([1, 2, 3, 4, 5, 6, 7, 8]);
        const [result] = await zaplib.callRustAsync("total_sum", [data]);
        expect(result, "36");
      },
      "Call Rust (with ZapBuffer)": async () => {
        const buffer = zaplib.createReadOnlyBuffer(
          new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8])
        );
        const [result] = await zaplib.callRustAsync<[Uint8Array]>(
          "array_multiply_u8_readonly",
          [JSON.stringify(10), buffer]
        );
        expect(result.length, 8);
        expect(result[0], 10);
        expect(result[1], 20);
        expect(result[2], 30);
        expect(result[3], 40);
        expect(result[4], 50);
        expect(result[5], 60);
        expect(result[6], 70);
        expect(result[7], 80);
        return Promise.all([
          expectDeallocationOrUnregister(buffer),
          expectDeallocationOrUnregister(result),
        ]);
      },
      "Call Rust (with Mutable ZapBuffer)": async () => {
        // TODO(Paras): Add enforcement of readonly ZapArrays and test it.
        // const [buffer] = await zaplib.callRustAsync("make_zapbuffer");
        // let err;
        // try {
        //     buffer[0] = 0;
        // } catch (e) {
        //     err = e;
        // } finally {
        //     expect(err?.message, "Cannot mutate a read-only array");
        // }

        const mutableBuffer = await zaplib.createMutableBuffer(
          new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8])
        );
        expect(mutableBuffer.length, 8);
        expect(mutableBuffer[0], 1);
        expect(mutableBuffer[1], 2);
        expect(mutableBuffer[2], 3);
        expect(mutableBuffer[3], 4);
        expect(mutableBuffer[4], 5);
        expect(mutableBuffer[5], 6);
        expect(mutableBuffer[6], 7);
        expect(mutableBuffer[7], 8);

        // Mutate the buffer to ensure the changes are detected in Rust code
        mutableBuffer[0] = 0;
        mutableBuffer[1] = 0;
        mutableBuffer[2] = 0;
        mutableBuffer[3] = 0;

        const [result] = await zaplib.callRustAsync<[Uint8Array]>(
          "array_multiply_u8",
          [JSON.stringify(10), mutableBuffer]
        );
        expect(result.length, 8);
        expect(result[0], 0);
        expect(result[1], 0);
        expect(result[2], 0);
        expect(result[3], 0);
        expect(result[4], 50);
        expect(result[5], 60);
        expect(result[6], 70);
        expect(result[7], 80);

        return Promise.all([
          expectDeallocationOrUnregister(mutableBuffer),
          expectDeallocationOrUnregister(result),
        ]);
      },
      "Call Rust with Float32Array": () => {
        // Using a normal array
        const input = new Float32Array([0.1, 0.9, 0.3]);
        const [result] = zaplib.callRustSync<[Float32Array]>(
          "array_multiply_f32",
          [JSON.stringify(10), input]
        );
        expect(result.length, 3);
        expect(result[0], 1);
        expect(result[1], 9);
        expect(result[2], 3);

        // Using a ZapArray
        const input2 = zaplib.createMutableBuffer(
          new Float32Array([0.1, 0.9, 0.3])
        );
        const [result2] = zaplib.callRustSync<[Float32Array]>(
          "array_multiply_f32",
          [JSON.stringify(10), input2]
        );

        expect(result2.length, 3);
        expect(result2[0], 1);
        expect(result2[1], 9);
        expect(result2[2], 3);

        // Using a readonly ZapArray
        const input3 = zaplib.createReadOnlyBuffer(
          new Float32Array([0.1, 0.9, 0.3])
        );

        const [result3] = zaplib.callRustSync<[Float32Array]>(
          "array_multiply_f32_readonly",
          [JSON.stringify(10), input3]
        );

        expect(result3.length, 3);
        expect(result3[0], 1);
        expect(result3[1], 9);
        expect(result3[2], 3);

        return Promise.all([
          expectDeallocationOrUnregister(result),
          expectDeallocationOrUnregister(input2),
          expectDeallocationOrUnregister(result2),
          expectDeallocationOrUnregister(input3),
          expectDeallocationOrUnregister(result3),
        ]);
      },
      "Call Rust (in same thread)": () => {
        const buffer = new SharedArrayBuffer(8);
        new Uint8Array(buffer).set([1, 2, 3, 4, 5, 6, 7, 8]);
        const uint8Part = new Uint8Array(buffer, 2, 4);
        const [result] = zaplib.callRustSync("array_multiply_u8", [
          JSON.stringify(10),
          uint8Part,
        ]);
        expect(result.length, 4);
        expect(result[0], 30);
        expect(result[1], 40);
        expect(result[2], 50);
        expect(result[3], 60);
      },
      "Call Rust with Float32Array (in same thread)": () => {
        // Using a normal array
        const input = new Float32Array([0.1, 0.9, 0.3]);
        const [result] = zaplib.callRustSync("array_multiply_f32", [
          JSON.stringify(10),
          input,
        ]);
        expect(result.length, 3);
        expect(result[0], 1);
        expect(result[1], 9);
        expect(result[2], 3);

        // Using a ZapArray
        const input2 = zaplib.createMutableBuffer(
          new Float32Array([0.1, 0.9, 0.3])
        );
        const [result2] = zaplib.callRustSync("array_multiply_f32", [
          JSON.stringify(10),
          input2,
        ]);
        expect(result2.length, 3);
        expect(result2[0], 1);
        expect(result2[1], 9);
        expect(result2[2], 3);

        // Using a readonly ZapArray
        const input3 = zaplib.createReadOnlyBuffer(
          new Float32Array([0.1, 0.9, 0.3])
        );

        const [result3] = zaplib.callRustSync("array_multiply_f32_readonly", [
          JSON.stringify(10),
          input3,
        ]);
        expect(result3.length, 3);
        expect(result3[0], 1);
        expect(result3[1], 9);
        expect(result3[2], 3);
      },
      "Cast WrBuffers":  () => {
        const input = zaplib.createMutableBuffer(new Float32Array([0.1]));
        const castArray = new Uint8Array(input.buffer);
        expect(castArray.length, 4);
        expect(castArray[0], 205);
        expect(castArray[1], 204);
        expect(castArray[2], 204);
        expect(castArray[3], 61);
        expectThrow(
          () => zaplib.callRustSync("verify_cast_array", [castArray]),
          "Cannot call Rust with a buffer which has been cast to a different type. Expected F32Buffer but got U8Buffer"
        );

        const input2 = zaplib.createReadOnlyBuffer(new Float32Array([0.1]));
        const castArray2 = new Uint8Array(input2.buffer);
        expect(castArray2.length, 4);
        expect(castArray2[0], 205);
        expect(castArray2[1], 204);
        expect(castArray2[2], 204);
        expect(castArray2[3], 61);
        expectThrow(
          () => zaplib.callRustSync("verify_cast_array", [castArray2]),
          "Cannot call Rust with a buffer which has been cast to a different type. Expected ReadOnlyF32Buffer but got ReadOnlyU8Buffer"
        );
      },
      "On the main thread inWorker returns false": () => {
        expect(inWorker, false);
      },
      ...runtimeSpecificTests,
      ...zapBufferTests,
    };

    const checkWasmOffline = async () => {
      const asyncFuncs = [() => zaplib.callRustAsync("call_rust_no_return")];
      for (const f of asyncFuncs) {
        await expectThrowAsync(f, "Zaplib WebAssembly instance crashed");
      }
      const syncFuncs = [
        () => zaplib.createMutableBuffer(new Uint8Array()),
        () => zaplib.createReadOnlyBuffer(new Uint8Array()),
        () => {
          zaplib.callRustSync("call_rust_no_return");
        },
      ];
      for (const f of syncFuncs) {
        expectThrow(f, "Zaplib WebAssembly instance crashed");
      }

      await rpc.send("runTest", "testErrorAfterPanic");
    };

    const otherTests =
      zaplib.jsRuntime === "wasm"
        ? {
            "Disable RPC after panic": async () => {
              await expectThrowAsync(
                async () => {
                  await zaplib.callRustAsync("panic");
                },
                // TODO(Paras): An exact line number here is kind of annoying. Later we can have some sort of partial matcher.
                "panicked at 'I am panicking!', zaplib/web/test_suite/src/main.rs:109:17"
              );

              await checkWasmOffline();
            },
            "Throw error from event handling to user provided callback":
              async () => {
                await zaplib.callRustAsync("panic_signal");

                // TODO(Paras): Since event handling happens in a setTimeout, we have
                // to do this check some time after `callRustAsync`. For now, use a 10ms delay.
                setTimeout(async () => {
                  expect(onPanicCalled, true);
                  await checkWasmOffline();
                }, 10);
              },
            "Throw error from draw to user provided callback": async () => {
              await zaplib.callRustAsync("panic_draw");

              // TODO(Paras): Since event handling happens in a setTimeout, we have
              // to do this check some time after `callRustAsync`. For now, use a 10ms delay.
              setTimeout(async () => {
                expect(onPanicCalled, true);
                await checkWasmOffline();
              }, 10);
            },
          }
        : {};

    const makeButtons = () => {
      const jsRoot = assertNotNull(document.getElementById("root"));

      window.runAllTests3x = async () => {
        setInTest(true);
        for (let i = 0; i < 3; i++) {
          for (const [testName, test] of Object.entries(tests)) {
            console.log(`Running test: ${testName}`);
            await test();
            console.log(`✅ Success`);
            const button = document.getElementById(testName);
            if (button) {
              button.innerText += "✅";
            }
          }
        }
        console.log(
          `✅ All tests completed (3x to ensure no memory corruption!)`
        );
        setInTest(false);
      };
      const runAllButton = document.createElement("button");
      runAllButton.innerText = "Run All Tests 3x";
      runAllButton.onclick = window.runAllTests3x;
      const buttonDiv = document.createElement("div");
      buttonDiv.append(runAllButton);
      jsRoot.append(buttonDiv);

      for (const [name, test] of Object.entries(tests)) {
        const button = document.createElement("button");
        button.innerText = name;
        button.id = name;
        button.onclick = async () => {
          setInTest(true);
          console.log(`Running test: ${name}`);
          await test();
          console.log(`✅ Success`);
          button.innerText += "✅";
          setInTest(false);
        };

        const buttonDiv = document.createElement("div");
        buttonDiv.append(button);
        jsRoot.append(buttonDiv);
      }

      const otherTestsRoot = assertNotNull(
        document.getElementById("other-tests")
      );
      for (const [name, test] of Object.entries(otherTests)) {
        const button = document.createElement("button");
        button.innerText = name;
        button.onclick = async () => {
          setInTest(true);
          console.log(`Running test: ${name}`);
          await test();
          console.log(`✅ Success`);
          button.innerText += "✅";
          setInTest(false);
        };

        const buttonDiv = document.createElement("div");
        buttonDiv.append(button);
        otherTestsRoot.append(buttonDiv);
      }
    };

    if (document.readyState !== "loading") {
      makeButtons();
    } else {
      document.addEventListener("DOMContentLoaded", makeButtons);
    }
  });
