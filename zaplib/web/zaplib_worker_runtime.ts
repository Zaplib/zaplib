/// <reference lib="WebWorker" />

// The "Zaplib WebWorker runtime" exposes some common Zaplib functions inside your WebWorkers, like `callRustAsync`.
//
// Include the output of this (zaplib_worker_runtime.js) at the start of each worker, and initialize the runtime
// by calling `self.initializeWorker` with a `MessagePort` obtained by `newWorkerPort` (which is
// available on `window` in the main browser thread, and in any worker that already has the runtime running). You
// can pass the port to the worker using `postMessage`; just be sure to include it in the list of transferables.
//
// Currently this is only supported in WebAssembly, not when using CEF.

import {
  callRustSyncImpl,
  createErrorCheckers,
  createWasmBuffer,
  getWasmEnv,
  getZapParamType,
  initThreadLocalStorageAndStackOtherWorkers,
  Rpc,
  transformParamsFromRustImpl,
} from "common";
import { MainWorkerChannelEvent, WebWorkerRpc } from "rpc_types";
import {
  CallRustAsync,
  CallRustSync,
  PostMessageTypedArray,
  WasmExports,
  ZapArray,
  RustZapParam,
  MutableBufferData,
  CreateBufferWorkerSync,
} from "types";
import { inWorker } from "type_of_runtime";
import {
  getZapBufferWasm,
  isZapBuffer,
  overwriteTypedArraysWithZapArrays,
  unregisterMutableBuffer,
  ZapBuffer,
  checkValidZapArray,
} from "zap_buffer";

let rpc: Rpc<WebWorkerRpc>;
let wasmExports: WasmExports;
let wasmMemory: WebAssembly.Memory;
let wasmAppPtr: BigInt;

let alreadyCalledInitialize = false;

let wasmOnline: Uint8Array;
const wasmInitialized = () => Atomics.load(wasmOnline, 0) === 1;
const { checkWasm, wrapWasmExports } = createErrorCheckers(wasmInitialized);

export const initializeWorker = (zapWorkerPort: MessagePort): Promise<void> => {
  if (alreadyCalledInitialize) {
    throw new Error("Only call zaplib.initializeWorker once");
  }
  alreadyCalledInitialize = true;

  if (!inWorker) {
    throw new Error(
      "zaplib.initializeWorker() can only be called in a WebWorker"
    );
  }

  overwriteTypedArraysWithZapArrays();

  return new Promise((resolve) => {
    rpc = new Rpc(zapWorkerPort);

    rpc
      .send(MainWorkerChannelEvent.Init)
      .then(
        ({
          wasmModule,
          memory,
          taskWorkerSab,
          baseUri,
          appPtr,
          tlsAndStackData,
          wasmOnline: _wasmOnline,
        }) => {
          wasmOnline = _wasmOnline;
          wasmMemory = memory;
          wasmAppPtr = appPtr;

          function getExports() {
            return wasmExports;
          }

          const env = getWasmEnv({
            getExports,
            memory,
            taskWorkerSab,
            fileHandles: [], // TODO(JP): implement at some point..
            sendEventFromAnyThread: (eventPtr: BigInt) => {
              rpc.send(MainWorkerChannelEvent.SendEventFromAnyThread, eventPtr);
            },
            threadSpawn: () => {
              throw new Error("Not yet implemented");
            },
            baseUri,
          });

          WebAssembly.instantiate(wasmModule, { env }).then((instance: any) => {
            initThreadLocalStorageAndStackOtherWorkers(
              instance.exports,
              tlsAndStackData
            );
            wasmExports = wrapWasmExports(instance.exports);
            resolve();
          });
        }
      );
  });
};

const destructor = (arcPtr: number) => {
  wasmExports.decrementArc(BigInt(arcPtr));
};

const mutableDestructor = ({
  bufferPtr,
  bufferLen,
  bufferCap,
}: MutableBufferData) => {
  wasmExports.deallocVec(
    BigInt(bufferPtr),
    BigInt(bufferLen),
    BigInt(bufferCap)
  );
};

const transformParamsFromRust = (params: RustZapParam[]) =>
  transformParamsFromRustImpl(
    wasmMemory,
    destructor,
    mutableDestructor,
    params
  );

export const newWorkerPort = (): MessagePort => {
  const channel = new MessageChannel();
  rpc.send(MainWorkerChannelEvent.BindMainWorkerPort, channel.port1, [
    channel.port1,
  ]);
  return channel.port2;
};

// TODO(JP): Allocate buffers on the wasm memory directly here.
export const callRustAsync: CallRustAsync = async (name, params = []) => {
  checkWasm();

  const transformedParams = params.map((param) => {
    if (typeof param === "string") {
      return param;
    } else if (isZapBuffer(param.buffer)) {
      checkValidZapArray(param);
      return serializeZapArrayForPostMessage(param);
    } else {
      if (!(param.buffer instanceof SharedArrayBuffer)) {
        console.warn(
          "Consider passing Uint8Arrays backed by ZapBuffer or SharedArrayBuffer into `callRustAsync` to prevent copying data"
        );
      }
      return param;
    }
  });

  return transformParamsFromRust(
    await rpc.send(MainWorkerChannelEvent.CallRustAsync, {
      name,
      params: transformedParams,
    })
  );
};

export const callRustSync: CallRustSync = (name, params = []) =>
  callRustSyncImpl({
    name,
    params,
    checkWasm,
    wasmMemory,
    wasmExports,
    wasmAppPtr,
    transformParamsFromRust,
  });

// TODO(JP): See comment at CreateBufferWorkerSync type.
export const createMutableBuffer: CreateBufferWorkerSync = (data) => {
  checkWasm();

  const bufferLen = data.byteLength;
  const bufferPtr = createWasmBuffer(wasmMemory, wasmExports, data);
  return transformParamsFromRust([
    {
      paramType: getZapParamType(data, false),
      bufferPtr,
      bufferLen,
      bufferCap: bufferLen,
      readonly: false,
    },
  ])[0] as typeof data;
};

// TODO(JP): See comment at CreateBufferWorkerSync type.
export const createReadOnlyBuffer: CreateBufferWorkerSync = (data) => {
  checkWasm();

  const bufferPtr = createWasmBuffer(wasmMemory, wasmExports, data);
  const paramType = getZapParamType(data, true);
  const arcPtr = Number(
    wasmExports.createArcVec(
      BigInt(bufferPtr),
      BigInt(data.length),
      BigInt(paramType)
    )
  );

  return transformParamsFromRust([
    {
      paramType,
      bufferPtr,
      bufferLen: data.byteLength,
      arcPtr,
      readonly: true,
    },
  ])[0] as typeof data;
};

// TODO(JP): Somewhat duplicated with the other implementation.
export const serializeZapArrayForPostMessage = (
  zapArray: ZapArray
): PostMessageTypedArray => {
  if (!(typeof zapArray === "object" && isZapBuffer(zapArray.buffer))) {
    throw new Error("Only pass Zap arrays to serializeZapArrayForPostMessage");
  }
  const zapBuffer = zapArray.buffer as ZapBuffer;
  if (zapBuffer.__zaplibBufferData.readonly) {
    wasmExports.incrementArc(BigInt(zapBuffer.__zaplibBufferData.arcPtr));
  } else {
    unregisterMutableBuffer(zapBuffer);
  }
  return {
    bufferData: zapBuffer.__zaplibBufferData,
    byteOffset: zapArray.byteOffset,
    byteLength: zapArray.byteLength,
  };
};

export const deserializeZapArrayFromPostMessage = (
  postMessageData: PostMessageTypedArray
): Uint8Array => {
  const zapBuffer = getZapBufferWasm(
    wasmMemory,
    postMessageData.bufferData,
    destructor,
    mutableDestructor
  );
  return new Uint8Array(
    zapBuffer,
    postMessageData.byteOffset,
    postMessageData.byteLength
  );
};
