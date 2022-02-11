// @ts-ignore
import TestSuiteWorker from "worker-loader?inline=no-fallback!./test_suite_worker";

import { assertNotNull, Rpc } from "./common";
import { TestSuiteTests } from "./test_suite_worker";
import { PostMessageTypedArray, ZapArray } from "./types";
import { zapBufferTests } from "./zap_buffer_test";
import * as zaplib from "./zaplib_runtime";
import {
  expect,
  expectDeallocationOrUnregister as _expectDeallocationOrUnregister,
  expectThrowAsync,
  setInTest,
} from "./test_helpers";
import { inWorker } from "./type_of_runtime";

const expectDeallocationOrUnregister = (buffer: ZapArray) =>
  _expectDeallocationOrUnregister(zaplib.callRust, buffer);

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
    testCallRustInSameThreadSyncWithZapbuffer: [void, PostMessageTypedArray];
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

zaplib
  .initialize({
    wasmModule: `target/wasm32-unknown-unknown/${env}/test_suite.wasm`,
    defaultStyles: true,
  })
  .then(async () => {
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
            "Call rust from worker": runWorkerTest("testCallRustFromWorker"),
            "Call rust (no return) from worker": runWorkerTest(
              "testCallRustNoReturnFromWorker"
            ),
            "Call rust with Float32Array from worker": runWorkerTest(
              "testCallRustFloat32ArrayFromWorker"
            ),
            "Call rust in same thread sync with Float32Array from worker":
              runWorkerTest(
                "testCallRustInSameThreadSyncFloat32ArrayFromWorker"
              ),
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
            "Call Rust in same thread with zapbuffer": async () => {
              const result = await rpc.send(
                "testCallRustInSameThreadSyncWithZapbuffer"
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
              "testCallRustInSameThreadSyncWithSignal"
            ),
          }
        : {
            "Call Rust (in same thread)": () => {
              const buffer = new SharedArrayBuffer(8);
              new Uint8Array(buffer).set([1, 2, 3, 4, 5, 6, 7, 8]);
              const uint8Part = new Uint8Array(buffer, 2, 4);
              const [result] = zaplib.callRustInSameThreadSync(
                "array_multiply_u8",
                [JSON.stringify(10), uint8Part]
              );
              expect(result.length, 4);
              expect(result[0], 30);
              expect(result[1], 40);
              expect(result[2], 50);
              expect(result[3], 60);
            },
            "Call Rust with Float32Array (in same thread)": async () => {
              // Using a normal array
              const input = new Float32Array([0.1, 0.9, 0.3]);
              const result = zaplib.callRustInSameThreadSync(
                "array_multiply_f32",
                [JSON.stringify(10), input]
              )[0] as Float32Array;
              expect(result.length, 3);
              expect(result[0], 1);
              expect(result[1], 9);
              expect(result[2], 3);

              // Using a ZapArray
              const input2 = await zaplib.createMutableBuffer(
                new Float32Array([0.1, 0.9, 0.3])
              );
              const result2 = zaplib.callRustInSameThreadSync(
                "array_multiply_f32",
                [JSON.stringify(10), input2]
              )[0] as Float32Array;
              expect(result2.length, 3);
              expect(result2[0], 1);
              expect(result2[1], 9);
              expect(result2[2], 3);

              // Using a readonly ZapArray
              const input3 = await zaplib.createReadOnlyBuffer(
                new Float32Array([0.1, 0.9, 0.3])
              );

              const result3 = zaplib.callRustInSameThreadSync(
                "array_multiply_f32_readonly",
                [JSON.stringify(10), input3]
              )[0] as Float32Array;
              expect(result3.length, 3);
              expect(result3[0], 1);
              expect(result3[1], 9);
              expect(result3[2], 3);
            },
          };

    const tests = {
      "Call Rust": async () => {
        const buffer = new SharedArrayBuffer(8);
        new Uint8Array(buffer).set([1, 2, 3, 4, 5, 6, 7, 8]);
        const uint8Part = new Uint8Array(buffer, 2, 4);
        const [result] = await zaplib.callRust("array_multiply_u8", [
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
        const result = await zaplib.callRust("call_rust_no_return");
        expect(result.length, 0);
      },
      "Call Rust (string return)": async () => {
        const buffer = new SharedArrayBuffer(8);
        const data = new Uint8Array(buffer);
        data.set([1, 2, 3, 4, 5, 6, 7, 8]);
        const [result] = await zaplib.callRust("total_sum", [data]);
        expect(result, "36");
      },
      "Call Rust (with ZapBuffer)": async () => {
        const buffer = await zaplib.createReadOnlyBuffer(
          new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8])
        );
        const result = (
          await zaplib.callRust("array_multiply_u8_readonly", [
            JSON.stringify(10),
            buffer,
          ])
        )[0] as Uint8Array;
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
        // const [buffer] = await zaplib.callRust("make_zapbuffer");
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

        const result = (
          await zaplib.callRust("array_multiply_u8", [
            JSON.stringify(10),
            mutableBuffer,
          ])
        )[0] as Uint8Array;
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
      "Call Rust with Float32Array": async () => {
        // Using a normal array
        const input = new Float32Array([0.1, 0.9, 0.3]);
        const result = (
          await zaplib.callRust("array_multiply_f32", [
            JSON.stringify(10),
            input,
          ])
        )[0] as Float32Array;
        expect(result.length, 3);
        expect(result[0], 1);
        expect(result[1], 9);
        expect(result[2], 3);

        // Using a ZapArray
        const input2 = await zaplib.createMutableBuffer(
          new Float32Array([0.1, 0.9, 0.3])
        );
        const result2 = (
          await zaplib.callRust("array_multiply_f32", [
            JSON.stringify(10),
            input2,
          ])
        )[0] as Float32Array;

        expect(result2.length, 3);
        expect(result2[0], 1);
        expect(result2[1], 9);
        expect(result2[2], 3);

        // Using a readonly ZapArray
        const input3 = await zaplib.createReadOnlyBuffer(
          new Float32Array([0.1, 0.9, 0.3])
        );

        const result3 = (
          await zaplib.callRust("array_multiply_f32_readonly", [
            JSON.stringify(10),
            input3,
          ])
        )[0] as Float32Array;

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
      "Cast WrBuffers": async () => {
        const input = await zaplib.createMutableBuffer(new Float32Array([0.1]));
        const castArray = new Uint8Array(input.buffer);
        expect(castArray.length, 4);
        expect(castArray[0], 205);
        expect(castArray[1], 204);
        expect(castArray[2], 204);
        expect(castArray[3], 61);
        await expectThrowAsync(
          () => zaplib.callRust("verify_cast_array", [castArray]),
          "Cannot call Rust with a buffer which has been cast to a different type. Expected F32Buffer but got U8Buffer"
        );

        const input2 = await zaplib.createReadOnlyBuffer(
          new Float32Array([0.1])
        );
        const castArray2 = new Uint8Array(input2.buffer);
        expect(castArray2.length, 4);
        expect(castArray2[0], 205);
        expect(castArray2[1], 204);
        expect(castArray2[2], 204);
        expect(castArray2[3], 61);
        await expectThrowAsync(
          () => zaplib.callRust("verify_cast_array", [castArray2]),
          "Cannot call Rust with a buffer which has been cast to a different type. Expected ReadOnlyF32Buffer but got ReadOnlyU8Buffer"
        );
      },
      "On the main thread inWorker returns false": () => {
        expect(inWorker, false);
      },
      ...runtimeSpecificTests,
      ...zapBufferTests,
    };

    const otherTests =
      zaplib.jsRuntime === "wasm"
        ? {
            "Disable RPC after panic": async () => {
              await expectThrowAsync(
                async () => {
                  await zaplib.callRust("panic");
                },
                // TODO(Paras): An exact line number here is kind of annoying. Later we can have some sort of partial matcher.
                "panicked at 'I am panicking!', zaplib/test_suite/src/main.rs:109:17"
              );

              // all calls to Rust should fail after this
              const funcs = [
                () => zaplib.callRust("call_rust_no_return"),
                () => zaplib.createMutableBuffer(new Uint8Array()),
                () => zaplib.createReadOnlyBuffer(new Uint8Array()),
              ];
              for (const f of funcs) {
                await expectThrowAsync(
                  f,
                  "Zaplib WebAssembly instance crashed"
                );
              }

              await rpc.send("runTest", "testErrorAfterPanic");
            },
          }
        : {};

    const makeButtons = () => {
      const jsRoot = assertNotNull(document.getElementById("root"));

      const runAllButton = document.createElement("button");
      runAllButton.innerText = "Run All Tests 3x";
      runAllButton.onclick = async () => {
        setInTest(true);
        for (let i = 0; i < 3; i++) {
          for (const [testName, test] of Object.entries(tests)) {
            console.log(`Running test: ${testName}`);
            await test();
            console.log(`✅ Success`);
          }
        }
        console.log(
          `✅ All tests completed (3x to ensure no memory corruption!)`
        );
        setInTest(false);
      };
      const buttonDiv = document.createElement("div");
      buttonDiv.append(runAllButton);
      jsRoot.append(buttonDiv);

      for (const [name, test] of Object.entries(tests)) {
        const button = document.createElement("button");
        button.innerText = name;
        button.onclick = async () => {
          setInTest(true);
          console.log(`Running test: ${name}`);
          await test();
          console.log(`✅ Success`);
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
