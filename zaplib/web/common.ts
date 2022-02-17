// This file should only be imported by WebWorkers
/// <reference lib="WebWorker" />

import { RpcSpec } from "rpc_types";
import {
  FileHandle,
  MutableBufferData,
  RustZapParam,
  TlsAndStackData,
  WasmEnv,
  WasmExports,
  ZapArray,
  ZapParamType,
} from "types";
import { getCachedZapBuffer, getZapBufferWasm } from "zap_buffer";
import { ZerdeBuilder } from "zerde";

////////////////////////////////////////////////////////////////
// RPC
////////////////////////////////////////////////////////////////

// Taken from https://github.com/cruise-automation/webviz/blob/6a4226bc2959444704d650d8c55cea4f4220c75c/packages/webviz-core/src/util/Rpc.js
// TODO(JP): Maybe release as a package?
// TODO(JP): Also be sure to include the tests at some point: https://github.com/cruise-automation/webviz/blob/6a4226bc2959444704d650d8c55cea4f4220c75c/packages/webviz-core/src/util/Rpc.test.js

// this type mirrors the MessageChannel and MessagePort APIs which are available on
// instances of web-workers and shared-workers respectively, as well as avaiable on
// 'global' within them.
export interface Channel {
  postMessage(data: unknown, transfer?: unknown[]): void;
  onmessage: null | ((ev: MessageEvent) => unknown);
}

const RESPONSE = "$$RESPONSE";
const ERROR = "$$ERROR";

// helper function to create linked channels for testing
function _createLinkedChannels(): { local: Channel; remote: Channel } {
  const local: Channel = {
    onmessage,

    postMessage(data: unknown, _transfer?: Array<ArrayBuffer>) {
      const ev = new MessageEvent("message", { data });
      if (remote.onmessage) {
        remote.onmessage(ev);
      }
    },
  };

  const remote: Channel = {
    onmessage,

    postMessage(data, _transfer) {
      const ev = new MessageEvent("message", { data });
      if (local.onmessage) {
        local.onmessage(ev);
      }
    },
  };
  return { local, remote };
}

// This class allows you to hook up bi-directional async calls across web-worker
// boundaries where a single call to or from a worker can 'wait' on the response.
// Errors in receivers are propigated back to the caller as a rejection.
// It also supports returning transferrables over the web-worker postMessage api,
// which was the main shortcomming with the worker-rpc npm module.
// To attach rpc to an instance of a worker in the main thread:
//   const rpc = new Rpc(workerInstace);
// To attach rpc within an a web worker:
//   const rpc = new Rpc(global);
// Check out the tests for more examples.
// See `rpc_types.ts` for descriptions of how to set up typed interactions.
export class Rpc<T extends RpcSpec> {
  static transferrables = "$$TRANSFERRABLES";
  _channel: Channel;
  _messageId = 0;
  _pendingCallbacks: Record<number, (arg0: any) => void> = {};
  _receivers = new Map<string, (value: any) => any>();

  constructor(channel: Channel) {
    this._channel = channel;
    if (this._channel.onmessage) {
      throw new Error(
        "channel.onmessage is already set. Can only use one Rpc instance per channel."
      );
    }
    this._channel.onmessage = this._onChannelMessage;
  }

  _onChannelMessage = (ev: MessageEvent): void => {
    const { id, topic, data } = ev.data as {
      topic: string;
      id: number;
      data: unknown;
    };
    if (topic === RESPONSE) {
      this._pendingCallbacks[id](ev.data);
      delete this._pendingCallbacks[id];
      return;
    }
    // invoke the receive handler in a promise so if it throws synchronously we can reject
    new Promise((resolve) => {
      const handler = this._receivers.get(topic);
      if (!handler) {
        throw new Error(`no receiver registered for ${topic}`);
      }
      // This works both when `handler` returns a value or a Promise.
      resolve(handler(data));
    })
      .then((result: any) => {
        if (!result) {
          this.postMessage({ topic: RESPONSE, id }, []);
          return;
        }
        const transferrables = result[Rpc.transferrables];
        delete result[Rpc.transferrables];
        const message = {
          topic: RESPONSE,
          id,
          data: result,
        };
        this.postMessage(message, transferrables);
      })
      .catch((err) => {
        const message = {
          topic: RESPONSE,
          id,
          data: {
            [ERROR]: true,
            name: err.name,
            message: err.message,
            stack: err.stack,
          },
        };
        this.postMessage(message, []);
      });
  };

  // send a message across the rpc boundary to a receiver on the other side
  // this returns a promise for the receiver's response.  If there is no registered
  // receiver for the given topic, this method throws
  send<U extends keyof T["send"]>(
    topic: U,
    data?: T["send"][U][0],
    transfer?: any[]
  ): Promise<T["send"][U][1]> {
    const id = this._messageId++;
    const message = { topic, id, data };
    const result = new Promise<any>((resolve, reject) => {
      this._pendingCallbacks[id] = (info: any) => {
        if (info.data && info.data[ERROR]) {
          const error = new Error(info.data.message);
          error.name = info.data.name;
          error.stack = info.data.stack;
          reject(error);
        } else {
          resolve(info.data);
        }
      };
    });
    this.postMessage(message, transfer);
    return result;
  }

  // register a receiver for a given message on a topic
  // only one receiver can be registered per topic and currently
  // 'deregistering' a receiver is not supported since this is not common
  receive<U extends keyof T["receive"] & string>(
    topic: U,
    handler: (arg0: T["receive"][U][0]) => T["receive"][U][1]
  ): void {
    if (this._receivers.has(topic)) {
      throw new Error(`Receiver already registered for topic: ${topic}`);
    }
    this._receivers.set(topic, handler);
  }

  private postMessage(message: unknown, transfer: unknown[] | undefined) {
    try {
      this._channel.postMessage(message, transfer);
    } catch (e) {
      console.error("Rpc postMessage call itself failed: ", e);
    }
  }
}

////////////////////////////////////////////////////////////////
// Mutex
////////////////////////////////////////////////////////////////

const MUTEX_UNLOCKED = 0;
const MUTEX_LOCKED = 1;

export const mutexLock = (sabi32: Int32Array, offset: number): void => {
  // This needs to be in a loop, because between the `wait` and `compareExchange` another thread might
  // take the Mutex.
  // eslint-disable-next-line no-constant-condition
  while (true) {
    if (
      Atomics.compareExchange(sabi32, offset, MUTEX_UNLOCKED, MUTEX_LOCKED) ==
      MUTEX_UNLOCKED
    ) {
      return;
    }
    Atomics.wait(sabi32, offset, MUTEX_LOCKED);
  }
};

export const mutexUnlock = (sabi32: Int32Array, offset: number): void => {
  if (
    Atomics.compareExchange(sabi32, offset, MUTEX_LOCKED, MUTEX_UNLOCKED) !=
    MUTEX_LOCKED
  ) {
    throw new Error("Called mutex_unlock on an already unlocked mutex");
  }
  Atomics.notify(sabi32, offset, 1);
};

////////////////////////////////////////////////////////////////
// Task worker
////////////////////////////////////////////////////////////////

export const TW_SAB_MUTEX_PTR = 0;
export const TW_SAB_MESSAGE_COUNT_PTR = 1;

// Initialize a SharedArrayBuffer used to communicate with task_worker.ts. This
// is a one-way communication channel; use pointers into `memory` for communicating
// information back.
//
// We use this because we typically can't use `postMessage`; see task_worker.ts
// for more details.
//
// Format:
// * i32 (4 bytes)         - read/write mutex
// * i32 (4 bytes)         - number of messages in queue (notify on this to wake up the task worker - it will
//                           read this before taking a mutex, but then reread it after taking the mutex)
// * n * u32 (n * 4 bytes) - pointers to messages serialized with `ZerdeBuilder`
export const initTaskWorkerSab = (): SharedArrayBuffer => {
  const bufferSizeBytes = 10000;
  const taskWorkerSab = new SharedArrayBuffer(bufferSizeBytes);
  const taskWorkerSabi32 = new Int32Array(taskWorkerSab);
  taskWorkerSabi32[TW_SAB_MUTEX_PTR] = MUTEX_UNLOCKED;
  taskWorkerSabi32[TW_SAB_MESSAGE_COUNT_PTR] = 0;
  return taskWorkerSab;
};

// Append a new message pointer to the SharedArrayBuffer used by task_worker.ts,
// and wake it up so it can process this new message (unless it's currently in polling
// mode, in that case the `Atomics.notify` will just not do anything).
const sendTaskWorkerMessage = (
  taskWorkerSab: SharedArrayBuffer,
  twMessagePtr: number
) => {
  const taskWorkerSabi32 = new Int32Array(taskWorkerSab);
  mutexLock(taskWorkerSabi32, TW_SAB_MUTEX_PTR);

  const currentNumberOfMessages = taskWorkerSabi32[TW_SAB_MESSAGE_COUNT_PTR];
  // Use unsigned numbers for the actual pointer, since they can be >2GB.
  new Uint32Array(taskWorkerSab)[currentNumberOfMessages + 2] = twMessagePtr;
  taskWorkerSabi32[TW_SAB_MESSAGE_COUNT_PTR] = currentNumberOfMessages + 1;

  mutexUnlock(taskWorkerSabi32, TW_SAB_MUTEX_PTR);
  Atomics.notify(taskWorkerSabi32, 1);
};

////////////////////////////////////////////////////////////////
// Wasm Thread initialization
////////////////////////////////////////////////////////////////

// Threads in WebAssembly! They are.. fun! Here's what happens.
//
// The first Wasm instance we start is in the main worker. It does the following:
// - It initializes static memory using `__wasm_init_memory`, which is automatically set
//   by LLVM as the special "start" function.
// - It already has memory allocated for the "shadow stack". This is like any stack in a
//   native program, but in WebAssembly it's called the "shadow stack" because WebAssembly
//   itself also has a notion of a stack built-in. It is however not suitable for all
//   kinds of data, which is why we need another separate stack.
// - We allocate Thread Local Storage (TLS) by allocating some memory on the heap (an
//   operation which by itself should not require TLS; otherwise we'd have a Catch-22
//   situation..), and calling `initThreadLocalStorageMainWorker` with it.
//
// Then, when we make any other WebAssembly threads (e.g. in our own WebWorkers, or in
// the WebWorkers of users), we do the following:
// - `__wasm_init_memory` is again called automatically, but will be skipped, since an
//   (atomic) flag has been set not to initialize static memory again.
// - We need to initialize memory for both the shadow stack and the Thread Local
//   Storage (TLS), using `makeThreadLocalStorageAndStackDataOnExistingThread`. We do this
//   by allocating memory on the heap on an already initialized thread, since allocating memory DOES
//   require the shadow stack to be initialized.
// - We then use this memory for both the TLS (on the lower side) and the shadow stack
//   (on the upper side, since it moves downward), using `initThreadLocalStorageAndStackOtherWorkers`.
//
// TODO(JP): This currently leaks memory since we never deallocate the TLS/shadow stack!
//
// TODO(JP): Even if we do deallocate the memory, there is currently no way to call TLS
// destructors; so we'd still leak memory. See https://github.com/rust-lang/rust/issues/77839

// The "shadow stack" size for new threads. Note that the main thread will
// keep using its own shadow stack size.
const WASM_STACK_SIZE_BYTES = 2 * 1024 * 1024; // 2 MB

// For the main worker, we only need to initialize Thread Local Storage (TLS).
export const initThreadLocalStorageMainWorker = (
  wasmExports: WasmExports
): void => {
  // Note that allocWasmMessage always aligns to 64 bits / 8 bytes.
  const ptr = wasmExports.allocWasmMessage(
    BigInt(wasmExports.__tls_size.value)
  );
  // TODO(JP): Cast to Number can cause trouble >2GB.
  wasmExports.__wasm_init_tls(Number(ptr));
};

// For non-main workers, we need to allocate enough data for Thread Local Storage (TLS)
// and the "shadow stack". We allocate this data in the main worker, and then send the
// pointer + size to other workers.
//
// This is easier than trying to allocate the appropriate amount of data in the other
// itself, which is possible (e.g. using memory.grow) but kind of cumbersome.
export const makeThreadLocalStorageAndStackDataOnExistingThread = (
  wasmExports: WasmExports
): TlsAndStackData => {
  // Align size to 64 bits / 8 bytes.
  const size =
    Math.ceil((wasmExports.__tls_size.value + WASM_STACK_SIZE_BYTES) / 8) * 8;
  // Note that allocWasmMessage always aligns to 64 bits / 8 bytes.
  const ptr = wasmExports.allocWasmMessage(BigInt(size));
  return { ptr, size };
};

// Set the shadow stack pointer and initialize thet Thread Local Storage (TLS).
//
// Note that the TLS sits on the lower side of the memory, wheras the shadow stack
// starts on the upper side of the memory and grows downwards.
//
// TODO(JP): __wasm_init_tls takes a Number, which might not work when it is >2GB.
export const initThreadLocalStorageAndStackOtherWorkers = (
  wasmExports: WasmExports,
  tlsAndStackData: TlsAndStackData
): void => {
  // Start the shadow stack pointer on the upper side of the memory, though subtract
  // 8 so we don't overwrite the byte right after the memory, while still keeping it
  // 64-bit aligned. TODO(JP): Is the 64-bit alignment necessary for the shadow stack?
  wasmExports.__stack_pointer.value =
    Number(tlsAndStackData.ptr) + tlsAndStackData.size - 8;
  wasmExports.__wasm_init_tls(
    // TODO(JP): Cast to Number can cause trouble >2GB.
    Number(tlsAndStackData.ptr)
  );
};

////////////////////////////////////////////////////////////////
// Common wasm functions
////////////////////////////////////////////////////////////////

export const copyArrayToRustBuffer = (
  inputBuffer: ZapArray,
  outputBuffer: ArrayBuffer,
  outputPtr: number
): void => {
  // should be the same type as inputBuffer
  // @ts-ignore: constructor is getting typed as Function instead of a constructor
  new inputBuffer.constructor(outputBuffer, outputPtr, inputBuffer.length).set(
    inputBuffer
  );
};

export const getZapParamType = (
  array: ZapArray,
  readonly: boolean
): ZapParamType => {
  if (array instanceof Uint8Array) {
    return readonly ? ZapParamType.ReadOnlyU8Buffer : ZapParamType.U8Buffer;
  } else if (array instanceof Float32Array) {
    return readonly ? ZapParamType.ReadOnlyF32Buffer : ZapParamType.F32Buffer;
  } else {
    throw new Error("Invalid array type");
  }
};

export const createWasmBuffer = (
  memory: WebAssembly.Memory,
  exports: WasmExports,
  data: ZapArray
): number => {
  const vecPtr = Number(exports.allocWasmVec(BigInt(data.byteLength)));
  copyArrayToRustBuffer(data, memory.buffer, vecPtr);
  return vecPtr;
};

export const makeZerdeBuilder = (
  memory: WebAssembly.Memory,
  wasmExports: WasmExports
): ZerdeBuilder => {
  const slots = 1024;
  // We have get memory.buffer *after* calling `allocWasmMessage`, because
  // there's a good chance it'll get swapped out (if it needed to grow the buffer).
  const byteOffset = Number(wasmExports.allocWasmMessage(BigInt(slots * 4)));
  return new ZerdeBuilder({
    buffer: memory.buffer,
    byteOffset: byteOffset,
    slots,
    growCallback: (_buffer, oldByteOffset, newBytes) => {
      const newByteOffset = Number(
        wasmExports.reallocWasmMessage(BigInt(oldByteOffset), BigInt(newBytes))
      );
      // We have get memory.buffer *after* calling `reallocWasmMessage`, because
      // there's a good chance it'll get swapped out (if it needed to grow the buffer).
      return { buffer: memory.buffer, byteOffset: newByteOffset };
    },
  });
};

export const getWasmEnv = ({
  getExports,
  memory,
  taskWorkerSab,
  fileHandles,
  sendEventFromAnyThread,
  threadSpawn,
  baseUri,
}: {
  getExports: () => WasmExports;
  memory: WebAssembly.Memory;
  taskWorkerSab: SharedArrayBuffer;
  fileHandles: FileHandle[];
  sendEventFromAnyThread: (eventPtr: BigInt) => void;
  threadSpawn: (ctxPtr: BigInt) => void;
  baseUri: string;
}): WasmEnv => {
  const parseString = (ptr: number, len: number) => {
    let out = "";
    // Can't use TextDecoder here since it doesn't work with SharedArrayBuffer.
    // TODO(JP): If it becomes important enough, we can see if making a copy to a regular
    // ArrayBuffer and then using TextDecoder is faster than what we do here.
    const array = new Uint32Array(memory.buffer, ptr, len);
    for (let i = 0; i < len; i++) {
      out += String.fromCharCode(array[i]);
    }
    return out;
  };

  return {
    memory,
    _consoleLog: (charsPtr, len) => {
      const out = parseString(parseInt(charsPtr), parseInt(len));
      console.log(out);
    },
    _throwError: (charsPtr, len) => {
      throw new RustPanic(parseString(parseInt(charsPtr), parseInt(len)));
    },
    readUserFileRange: (userFileId, bufPtr, bufLen, fileOffset) => {
      const file = fileHandles[userFileId];
      const start = Number(fileOffset);
      const end = start + Number(bufLen);
      if (file.lastReadStart <= start && start < file.lastReadEnd) {
        console.warn(
          `Read start (${start}) fell in the range of the last read (${file.lastReadStart}-${file.lastReadEnd}); ` +
            "this usually happens if you don't use BufReader or if you don't use BufReader.seek_relative."
        );
      }
      file.lastReadStart = start;
      file.lastReadEnd = end;
      // TODO(JP): This creates a new buffer instead of reading directly into the wasm memory.
      // Maybe we can avoid this by using a stream with a ReadableStreamBYOBReader, but that is
      // asynchronous, so we'd have to do a dance with another thread and atomics and all that,
      // and I don't know if that overhead would be worth it..
      const fileReaderSync = new FileReaderSync();
      const buffer = fileReaderSync.readAsArrayBuffer(
        file.file.slice(start, end)
      );
      copyArrayToRustBuffer(
        new Uint8Array(buffer),
        memory.buffer,
        Number(bufPtr)
      );
      return BigInt(buffer.byteLength);
    },
    performanceNow: () => {
      return performance.now();
    },
    threadSpawn: (ctxPtr) => {
      threadSpawn(ctxPtr);
    },
    _sendEventFromAnyThread: (eventPtr: BigInt) => {
      sendEventFromAnyThread(eventPtr);
    },
    readUrlSync: (urlPtr, urlLen, bufPtrOut, bufLenOut) => {
      const url = parseString(urlPtr, urlLen);
      const request = new XMLHttpRequest();
      request.responseType = "arraybuffer";
      request.open("GET", new URL(url, baseUri).href, false /* synchronous */);
      request.send(null);

      if (request.status === 200) {
        const exports = getExports();
        const outputBufPtr = createWasmBuffer(
          memory,
          exports,
          new Uint8Array(request.response)
        );
        new Uint32Array(memory.buffer, bufPtrOut, 1)[0] = outputBufPtr;
        new Uint32Array(memory.buffer, bufLenOut, 1)[0] =
          request.response.byteLength;
        return 1;
      } else {
        return 0;
      }
    },
    randomU64: () =>
      new BigUint64Array(
        self.crypto.getRandomValues(new Uint32Array(2)).buffer
      )[0],
    sendTaskWorkerMessage: (twMessagePtr) => {
      sendTaskWorkerMessage(taskWorkerSab, parseInt(twMessagePtr));
    },
  };
};

export function transformParamsFromRustImpl(
  memory: WebAssembly.Memory,
  destructor: (arcPtr: number) => void,
  mutableDestructor: (bufferData: MutableBufferData) => void,
  params: RustZapParam[]
): (string | ZapArray)[] {
  return params.map((param) => {
    if (typeof param === "string") {
      return param;
    } else {
      const zapBuffer = getZapBufferWasm(
        memory,
        param,
        destructor,
        mutableDestructor
      );

      if (param.paramType === ZapParamType.String) {
        throw new Error("ZapParam buffer type called with string paramType");
      }

      // These are actually ZapArray types, since we overwrite TypedArrays in overwriteTypedArraysWithZapArrays()
      const ArrayConstructor = {
        [ZapParamType.U8Buffer]: Uint8Array,
        [ZapParamType.ReadOnlyU8Buffer]: Uint8Array,
        [ZapParamType.F32Buffer]: Float32Array,
        [ZapParamType.ReadOnlyF32Buffer]: Float32Array,
      }[param.paramType];
      return getCachedZapBuffer(
        zapBuffer,
        new ArrayConstructor(
          zapBuffer,
          param.bufferPtr,
          param.bufferLen / ArrayConstructor.BYTES_PER_ELEMENT
        )
      );
    }
  });
}

export function assertNotNull<T>(
  value: T | null | undefined,
  objectName = "Value"
): T {
  if (value === null || value === undefined) {
    throw new Error(`Assertion failed: ${objectName} is null`);
  }
  return value;
}

export class RustPanic extends Error {
  constructor(message: string) {
    super(message);
    this.name = "RustPanic";
  }
}

export const createErrorCheckers = (
  wasmInitialized: () => boolean
): {
  checkWasm: () => void;
  wrapWasmExports: (wasmExports: WasmExports) => WasmExports;
} => {
  const checkWasm = () => {
    if (!wasmInitialized())
      throw new Error("Zaplib WebAssembly instance crashed");
  };

  return {
    checkWasm,
    wrapWasmExports: (exports: WasmExports) =>
      new Proxy(exports, {
        get: function (obj: WasmExports, prop: keyof WasmExports) {
          checkWasm();
          return obj[prop];
        },
      }),
  };
};
