// Wrapper around SharedArrayBuffer to encapsulate ownership of particular segments of it

import { getZapParamType } from "common";
import { BufferData, MutableBufferData, ZapArray, ZapParamType } from "types";
import { inTest } from "test_suite/test_helpers";

// TODO(Paras) - Make sure we monkeypatch on web workers as well
export class ZapBuffer extends SharedArrayBuffer {
  // This class supports both SharedArrayBuffer (wasm usecase) and ArrayBuffer (CEF)
  // In the future we can migrate to SharedArrayBuffer-s only once CEF supports those
  __zaplibWasmBuffer: SharedArrayBuffer | ArrayBuffer;
  __zaplibBufferData: BufferData;

  constructor(buffer: SharedArrayBuffer | ArrayBuffer, bufferData: BufferData) {
    super(0);
    this.__zaplibWasmBuffer = buffer;
    this.__zaplibBufferData = bufferData;
  }

  // TODO(Paras): Actually enforce this flag and prevent mutation of ZapArrays marked as readonly.
  // Potentially, we can do this by hashing read only buffer data and periodically checking in debug
  // builds if they have been modified/raising errors.
  get readonly(): boolean {
    return this.__zaplibBufferData.readonly;
  }

  // The only 2 methods on SharedArrayBuffer class to override:
  // See https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer#instance_properties
  get byteLength(): number {
    return this.__zaplibWasmBuffer.byteLength;
  }

  slice(...args: Parameters<SharedArrayBuffer["slice"]>): any {
    return this.__zaplibWasmBuffer.slice(...args);
  }
}

// This class is a drop-in replacement for all typed arrays
// It uses ZapBuffer as a handle for underlying buffer as the object that keeps underlying data around
// Requirements:
//  * The underlying typed array behaves like it was created over the original view
//  * When the new typed array (potentially with different class name) is created from the buffer of the original one,
//  they share the same handle
//
// The Rust side assumes that underlying data buffer is immutable,
// however it still could be accidentally modified on JS side leading to weird behavior
// TODO(Dmitry): Throw an error if there is mutation of the data
function zapBufferExtends(cls: any) {
  return class ZapTypedArray extends cls {
    constructor(...args: any) {
      const buffer = args[0];
      if (typeof buffer === "object" && buffer instanceof ZapBuffer) {
        // Fill in byteOffset if that's omitted.
        if (args.length < 2) {
          args[1] = buffer.__zaplibBufferData.bufferPtr;
        }
        // Fill in length (in elements, not in bytes) if that's omitted.
        if (args.length < 3) {
          args[2] = Math.floor(
            (buffer.__zaplibBufferData.bufferPtr +
              buffer.__zaplibBufferData.bufferLen -
              args[1]) /
              cls.BYTES_PER_ELEMENT
          );
        }
        if (args[1] < buffer.__zaplibBufferData.bufferPtr) {
          throw new Error(`Byte_offset ${args[1]} is out of bounds`);
        }
        if (
          args[1] + args[2] * cls.BYTES_PER_ELEMENT >
          buffer.__zaplibBufferData.bufferPtr +
            buffer.__zaplibBufferData.bufferLen
        ) {
          throw new Error(
            `Byte_offset ${args[1]} + length ${args[2]} is out of bounds`
          );
        }
        // Whenever we create ZapUintArray using ZapBuffer as first argument
        // pass the underlying full wasm_buffer further
        args[0] = buffer.__zaplibWasmBuffer;
        super(...args);
        this.__zaplibBuffer = buffer;
      } else {
        super(...args);
      }
    }

    get buffer() {
      return this.__zaplibBuffer || super.buffer;
    }

    subarray(begin = 0, end = this.length) {
      if (begin < 0) {
        begin = this.length + begin;
      }
      if (end < 0) {
        end = this.length + end;
      }
      if (end < begin) {
        end = begin;
      }
      return new ZapTypedArray(
        this.buffer,
        this.byteOffset + begin * this.BYTES_PER_ELEMENT,
        end - begin
      );
    }
  };
}

// Extending all typed arrays
// See https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects#indexed_collections
export const classesToExtend = {
  Int8Array: "ZapInt8Array",
  Uint8Array: "ZapUint8Array",
  Uint8ClampedArray: "ZapUint8ClampedArray",
  Int16Array: "ZapInt16Array",
  Uint16Array: "ZapUint16Array",
  Uint16ClampedArray: "ZapUint16ClampedArray",
  Int32Array: "ZapInt32Array",
  Uint32Array: "ZapUint32Array",
  Float32Array: "ZapFloat32Array",
  Float64Array: "ZapFloat64Array",
  BigInt64Array: "ZapBigInt64Array",
  BigUint64Array: "ZapBigUint64Array",
  DataView: "ZapDataView",
};

for (const [cls, zapCls] of Object.entries(classesToExtend)) {
  // Get a new type name by prefixing old one with "Zaplib".
  // e.g. Uint8Array is extended by ZapUint8Array, etc
  if (cls in self) {
    // @ts-ignore
    self[zapCls] = zapBufferExtends(self[cls]);
  }
}

// Checks if the given object itself or recursively contains ZapBuffers.
// Exported for tests.
export function containsZapBuffer(object: unknown): boolean {
  if (typeof object != "object" || object === null) {
    return false;
  }

  if (Object.prototype.hasOwnProperty.call(object, "__zaplibBuffer")) {
    return true;
  }

  // Only supporting nesting for arrays, plain objects, maps and sets similar to StructuredClone algorithm
  // See https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Structured_clone_algorithm#supported_types
  if (Array.isArray(object) || object instanceof Set || object instanceof Map) {
    for (const entry of object) {
      if (containsZapBuffer(entry)) {
        return true;
      }
    }
  } else if (Object.getPrototypeOf(object) === Object.getPrototypeOf({})) {
    for (const entry of Object.entries(object)) {
      if (containsZapBuffer(entry)) {
        return true;
      }
    }
  }
  return false;
}

function patchPostMessage(cls: any) {
  const origPostMessage = cls.postMessage;
  // Explicitly NOT a fat arrow (=>) since we want to keep the original `this`.
  cls.postMessage = function (...args: Parameters<Worker["postMessage"]>) {
    if (containsZapBuffer(args[0])) {
      // TODO(Dmitry): add a better error message showing the exact location of typed arrays
      throw new Error(
        "Sending ZapBuffers to/from workers is not supported - " +
          "use .slice() on typed array instead to make an explicit copy"
      );
    }
    origPostMessage.apply(this, args);
  };
}

export function overwriteTypedArraysWithZapArrays(): void {
  for (const [cls, zapCls] of Object.entries(classesToExtend)) {
    if (cls in self) {
      // @ts-ignore
      self[cls] = self[zapCls];
    }
  }
  patchPostMessage(self);

  // In Safari nested workers are not defined.
  if (self.Worker) {
    patchPostMessage(self.Worker);
  }

  // Skipping this in nodejs case as web-worker polyfill doesn't provide MessagePort
  if (self.MessagePort) {
    patchPostMessage(self.MessagePort);
  }
}

const zapBufferCache = new WeakMap<ZapBuffer, ZapArray>();
export function getCachedZapBuffer(
  zapBuffer: ZapBuffer,
  fallbackArray: ZapArray
): ZapArray {
  if (
    !(
      // Overwrite the cached value if we return a pointer to a buffer of a different type
      // For example, Rust code may cast a float to an u8 and return the same buffer pointer.
      (
        zapBufferCache.get(zapBuffer)?.BYTES_PER_ELEMENT ===
        fallbackArray.BYTES_PER_ELEMENT
      )
    )
  ) {
    zapBufferCache.set(zapBuffer, fallbackArray);
  }
  return zapBufferCache.get(zapBuffer) as ZapArray;
}

export function isZapBuffer(potentialZapBuffer: ArrayBufferLike): boolean {
  return (
    typeof potentialZapBuffer === "object" &&
    potentialZapBuffer instanceof ZapBuffer
  );
}

export function checkValidZapArray(zapArray: ZapArray): void {
  if (!isZapBuffer(zapArray.buffer)) {
    throw new Error("zapArray.buffer is not a ZapBuffer in checkValidZapArray");
  }
  const buffer = zapArray.buffer as ZapBuffer;

  const bufferCoversZapBuffer =
    zapArray.byteOffset === buffer.__zaplibBufferData.bufferPtr &&
    zapArray.byteLength === buffer.__zaplibBufferData.bufferLen;
  if (!bufferCoversZapBuffer) {
    throw new Error(
      "Called Rust with a buffer that does not span the entire underlying ZapBuffer"
    );
  }

  const paramType = getZapParamType(zapArray, buffer.readonly);
  if (paramType !== buffer.__zaplibBufferData.paramType) {
    throw new Error(
      `Cannot call Rust with a buffer which has been cast to a different type. Expected ${
        ZapParamType[buffer.__zaplibBufferData.paramType]
      } but got ${ZapParamType[paramType]}`
    );
  }
}

// Cache ZapBuffers so that we have a stable identity for ZapBuffers pointing to the same
// Arc. This is useful for any downstream caches in user code.
const bufferCache: { [arcPtr: number]: WeakRef<ZapBuffer> } = {};

export const allocatedArcs: Record<number, boolean> = {};
export const allocatedVecs: Record<number, boolean> = {};

const bufferRegistry = new FinalizationRegistry(
  ({
    arcPtr,
    destructor,
  }: {
    arcPtr: number;
    destructor?: (arcPtr: number) => void;
  }) => {
    if (inTest) {
      if (allocatedArcs[arcPtr] === false) {
        throw new Error(`Deallocating an already deallocated arcPtr ${arcPtr}`);
      } else if (allocatedArcs[arcPtr] === undefined) {
        throw new Error(`Deallocating an unallocated arcPtr ${arcPtr}`);
      }
      allocatedArcs[arcPtr] = false;
    }

    delete bufferCache[arcPtr];
    if (destructor) destructor(arcPtr);
  }
);

const mutableZapBufferRegistry = new FinalizationRegistry(
  ({
    bufferData,
    destructor,
  }: {
    bufferData: MutableBufferData;
    destructor: (bufferData: MutableBufferData) => void;
  }) => {
    if (inTest) {
      const { bufferPtr } = bufferData;
      if (allocatedVecs[bufferPtr] === false) {
        throw new Error(
          `Deallocating an already deallocated bufferPtr ${bufferPtr}`
        );
      } else if (allocatedVecs[bufferPtr] === undefined) {
        throw new Error(`Deallocating an unallocated bufferPtr ${bufferPtr}`);
      }
      allocatedVecs[bufferPtr] = false;
    }

    destructor(bufferData);
  }
);

// Return a buffer with a stable identity based on arcPtr.
// Register callbacks so we de-allocate the buffer when it goes out of scope.
export const getZapBufferWasm = (
  wasmMemory: WebAssembly.Memory,
  bufferData: BufferData,
  destructor: (arcPtr: number) => void,
  mutableDestructor: (bufferData: MutableBufferData) => void
): ZapBuffer => {
  if (bufferData.readonly) {
    if (!bufferCache[bufferData.arcPtr]?.deref()) {
      if (inTest) {
        allocatedArcs[bufferData.arcPtr] = true;
      }

      const zapBuffer = new ZapBuffer(wasmMemory.buffer, bufferData);

      bufferRegistry.register(zapBuffer, {
        arcPtr: bufferData.arcPtr,
        destructor,
        /* no unregisterToken here since we never need to unregister */
      });

      bufferCache[bufferData.arcPtr] = new WeakRef(zapBuffer);
    } else {
      // If we already hold a reference, decrement the Arc we were just given;
      // otherwise we leak memory.
      destructor(bufferData.arcPtr);
    }

    return bufferCache[bufferData.arcPtr].deref() as ZapBuffer;
  } else {
    if (inTest) {
      allocatedVecs[bufferData.bufferPtr] = true;
    }

    const zapBuffer = new ZapBuffer(wasmMemory.buffer, bufferData);

    mutableZapBufferRegistry.register(
      zapBuffer,
      {
        bufferData,
        destructor: mutableDestructor,
      },
      zapBuffer
    );

    return zapBuffer;
  }
};

// Remove mutable ZapBuffers without running destructors. This is useful
// when transferring ownership of buffers to Rust without deallocating data.
export const unregisterMutableBuffer = (zapBuffer: ZapBuffer): void => {
  if (zapBuffer.readonly) {
    throw new Error(
      "`unregisterMutableBuffer` should only be called on mutable ZapBuffers"
    );
  }

  mutableZapBufferRegistry.unregister(zapBuffer);

  if (inTest) {
    allocatedVecs[zapBuffer.__zaplibBufferData.bufferPtr] = false;
  }
};

// Return a buffer with a stable identity based on arcPtr
export const getZapBufferCef = (
  buffer: ArrayBuffer,
  arcPtr: number | undefined,
  paramType: ZapParamType
): ZapBuffer => {
  if (arcPtr) {
    if (!bufferCache[arcPtr]?.deref()) {
      const zapBuffer = new ZapBuffer(buffer, {
        bufferPtr: 0,
        bufferLen: buffer.byteLength,
        readonly: true,
        paramType,
        // TODO(Paras): These fields below do not apply to CEF
        arcPtr: -1,
      });

      bufferRegistry.register(zapBuffer, { arcPtr });
      bufferCache[arcPtr] = new WeakRef(zapBuffer);
    }
    return bufferCache[arcPtr].deref() as ZapBuffer;
  } else {
    return new ZapBuffer(buffer, {
      bufferPtr: 0,
      bufferLen: buffer.byteLength,
      bufferCap: buffer.byteLength,
      paramType,
      readonly: false,
    });
  }
};
