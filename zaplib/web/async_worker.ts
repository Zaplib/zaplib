import {
  Rpc,
  getWasmEnv,
  initThreadLocalStorageAndStackOtherWorkers,
  makeThreadLocalStorageAndStackDataOnExistingThread,
} from "./common";
import {
  AsyncWorkerRpc,
  Worker,
  AsyncWorkerEvent,
  MainWorkerChannelEvent,
} from "./rpc_types";
import { WasmExports } from "./types";

const rpc = new Rpc<Worker<AsyncWorkerRpc>>(self);

rpc.receive(
  AsyncWorkerEvent.Run,
  ({
    wasmModule,
    memory,
    taskWorkerSab,
    ctxPtr,
    fileHandles,
    baseUri,
    tlsAndStackData,
    mainWorkerPort,
  }) => {
    let exports: WasmExports;

    const mainThreadRpc = new Rpc(mainWorkerPort);

    const sendEventFromAnyThread = (eventPtr: BigInt) => {
      mainThreadRpc.send(
        MainWorkerChannelEvent.SendEventFromAnyThread,
        eventPtr
      );
    };
    const threadSpawn = (ctxPtr: BigInt) => {
      rpc.send(AsyncWorkerEvent.ThreadSpawn, {
        ctxPtr,
        tlsAndStackData:
          makeThreadLocalStorageAndStackDataOnExistingThread(exports),
      });
    };

    const getExports = () => {
      return exports;
    };
    const env = getWasmEnv({
      getExports,
      memory,
      taskWorkerSab,
      fileHandles,
      sendEventFromAnyThread,
      threadSpawn,
      baseUri,
    });

    return new Promise<void>((resolve, reject) => {
      WebAssembly.instantiate(wasmModule, { env }).then((instance) => {
        exports = instance.exports;
        initThreadLocalStorageAndStackOtherWorkers(exports, tlsAndStackData);
        // TODO(Paras): Eventually call `processWasmEvents` instead of a custom exported function.
        exports.runFunctionPointer(ctxPtr);
        resolve();
      }, reject);
    });
  }
);
