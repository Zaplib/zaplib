import { ZapParamType } from "../types";
import { ZapBuffer, classesToExtend, containsZapBuffer } from "../zap_buffer";
import { expect, expectThrow } from "./test_helpers";

declare global {
  interface Window {
    ZapUint8Array: typeof Uint8Array;
    ZapUint16Array: typeof Uint16Array;
  }
}

const { ZapUint8Array, ZapUint16Array } = window;

// Test that ZapArray is created like a DataView
function testBuffer(): void {
  const wasmMemory = new SharedArrayBuffer(1024);
  const buffer = new ZapBuffer(wasmMemory, {
    bufferPtr: 10,
    bufferLen: 4,
    bufferCap: 4,
    paramType: ZapParamType.U8Buffer,
    readonly: false,
  });
  const a = new ZapUint8Array(buffer, 10, 4);
  expect(a.byteOffset, 10);
  expect(a.length, 4);
}

// Test that new ZapArray shares the same ZapBuffer
function testShare(): void {
  const wasmMemory = new SharedArrayBuffer(1024);
  const buffer = new ZapBuffer(wasmMemory, {
    bufferPtr: 0,
    bufferLen: 1024,
    bufferCap: 1024,
    paramType: ZapParamType.U8Buffer,
    readonly: false,
  });
  const a = new ZapUint8Array(buffer);
  const b = new ZapUint16Array(a.buffer);
  expect(a.buffer, buffer);
  expect(a.buffer, b.buffer);
}

// Test ZapArray out-of-bounds behavior
function testOutOfBounds(): void {
  const wasmMemory = new SharedArrayBuffer(1024);
  const buffer = new ZapBuffer(wasmMemory, {
    bufferPtr: 1,
    bufferLen: 16,
    bufferCap: 16,
    paramType: ZapParamType.U8Buffer,
    readonly: false,
  });
  // start is outside of the view - should throw
  expectThrow(() => {
    new ZapUint8Array(buffer, 0);
  }, "Byte_offset 0 is out of bounds");

  // these doesn't throw but overwrites the end of the data
  const a = new ZapUint8Array(buffer, 1);
  expect(a.length, 16);
  const b = new ZapUint8Array(buffer, 2);
  expect(b.length, 15);

  // end is outside of the view - should throw
  expectThrow(() => {
    new ZapUint8Array(buffer, 15, 3);
  }, "Byte_offset 15 + length 3 is out of bounds");
}

// Test that ZapBuffer and ZapArray could be created from ArrayBuffer
function testArrayBuffer(): void {
  const array = new ArrayBuffer(16);
  const buffer = new ZapBuffer(array, {
    bufferPtr: 0,
    bufferLen: array.byteLength,
    bufferCap: array.byteLength,
    paramType: ZapParamType.U8Buffer,
    readonly: false,
  });
  const a = new ZapUint8Array(buffer);
  expect(a.byteOffset, 0);
  expect(a.byteLength, 16);
}

// Check that all names follow the convetion of having Zap as prefix
// e.g. ZapUint8Array overrides Uint8Array
function testZapNameMatches(): void {
  for (const [cls, zapCls] of Object.entries(classesToExtend)) {
    const expectedName = "Zap" + cls;
    expect(expectedName, zapCls);
  }
}

function testSubarray(): void {
  const wasmMemory = new SharedArrayBuffer(5);
  const regularArray = new Uint8Array(wasmMemory);
  regularArray.set(Uint8Array.from([0, 1, 2, 3, 4]));
  const buffer = new ZapBuffer(wasmMemory, {
    bufferPtr: 0,
    bufferLen: 5,
    bufferCap: 5,
    paramType: ZapParamType.U8Buffer,
    readonly: false,
  });
  const zapArray = new ZapUint8Array(buffer);

  expect(zapArray.subarray().buffer, buffer);
  expect(zapArray.subarray().toString(), regularArray.subarray().toString());
  expect(
    zapArray.subarray(1, 3).toString(),
    regularArray.subarray(1, 3).toString()
  );
  expect(
    zapArray.subarray(-2, 0).toString(),
    regularArray.subarray(-2, 0).toString()
  );
  expect(
    zapArray.subarray(-3, -1).toString(),
    regularArray.subarray(-3, -1).toString()
  );
  expect(
    zapArray.subarray(1, -1).toString(),
    regularArray.subarray(1, -1).toString()
  );
}

function testContainsZapBuffer(): void {
  const wasmMemory = new SharedArrayBuffer(16);
  const buffer = new ZapBuffer(wasmMemory, {
    bufferPtr: 0,
    bufferLen: 16,
    bufferCap: 16,
    paramType: ZapParamType.U8Buffer,
    readonly: false,
  });
  const a = new ZapUint8Array(buffer);

  expect(containsZapBuffer(a), true);
  expect(containsZapBuffer([a]), true);
  expect(containsZapBuffer({ key: a }), true);
  expect(containsZapBuffer(new Set([a])), true);

  const map = new Map();
  map.set("key", a);
  expect(containsZapBuffer(map), true);

  // calling slice removes the error
  expect(containsZapBuffer(a.slice()), false);

  // edge cases
  expect(containsZapBuffer(undefined), false);
  expect(containsZapBuffer(null), false);
}

export const zapBufferTests = {
  testBuffer,
  testShare,
  testOutOfBounds,
  testZapNameMatches,
  testArrayBuffer,
  testSubarray,
  testContainsZapBuffer,
};
