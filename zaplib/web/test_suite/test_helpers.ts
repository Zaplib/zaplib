import { CallRust, ZapArray } from "types";
import { jsRuntime } from "type_of_runtime";
import { allocatedArcs, allocatedVecs, ZapBuffer } from "zap_buffer";

export const expect = <T>(actual: T, expected: T): void => {
  if (expected === actual) {
    console.debug(`Success: Got ${actual}, Expected ${expected}`);
  } else {
    throw new Error(`Failure: Got ${actual}, Expected ${expected}`);
  }
};

// TODO(Paras): Would be nice to combine the two functions below at some point.
export const expectThrow = (f: () => void, expectedMessage?: string): void => {
  let error: Error | undefined;
  try {
    f();
  } catch (e: any) {
    error = e;
  }
  expect(!!error, true);
  if (error && expectedMessage) {
    expect(error.message, expectedMessage);
  }
};
export const expectThrowAsync = async (
  f: () => Promise<unknown>,
  expectedMessage?: string
): Promise<void> => {
  let error: Error | undefined;
  try {
    await f();
  } catch (e: any) {
    error = e;
  }
  expect(!!error, true);
  if (error && expectedMessage) {
    expect(error.message, expectedMessage);
  }
};

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

const checkConditionTimeout = async (
  condition: () => boolean,
  timeout: number
) => {
  const startTime = performance.now();
  while (!condition() && performance.now() < startTime + timeout) {
    await sleep(10);
  }
  return condition();
};

// Generate some dummy data and then delete it. This usually triggers the garbage collector.
const generateGarbage = () => {
  for (let i = 0; i < 10000; i++) {
    // @ts-ignore
    self["garbage_" + i] = { i };
  }
  for (let i = 0; i < 10000; i++) {
    // @ts-ignore
    delete self["garbage_" + i];
  }
};

const arcAllocated = async (callRust: CallRust, buffer: ZapBuffer) => {
  if (!buffer.__zaplibBufferData.readonly)
    throw new Error("arcAllocated called on mutable buffer");

  const arcPtr = buffer.__zaplibBufferData.arcPtr;

  // We still have the buffer here! So it should still be allocated.
  expect(allocatedArcs[arcPtr], true);

  const [result] = await callRust("check_arc_count", [`${BigInt(arcPtr)}`]);
  const [countBeforeDeallocation] = result;
  expect(countBeforeDeallocation, 1);

  return arcPtr;
};

const arcDeallocated = async (arcPtr: number) => {
  // From here on out we don't refer to `buffer` anymore, so it should get
  // deallocated, if the garbage collector is any good.
  expect(
    await checkConditionTimeout(() => {
      generateGarbage();
      return allocatedArcs[arcPtr] === false;
    }, 20000),
    true
  );
};

const vecDeallocated = async (bufferPtr: number) => {
  // Even though we have the buffer, it might have already been unregistered
  // when passed to Rust. We shouldn't read/write to it anymore. If this is the
  // case, let's just bail.
  if (!allocatedVecs[bufferPtr]) return;

  expect(
    await checkConditionTimeout(() => {
      generateGarbage();
      return allocatedVecs[bufferPtr] === false;
    }, 20000),
    true
  );
};

// Test that ZapBuffers were deallocated at some point in the next 20 seconds.
// This is a bit brittle given that there are no guarantees for garbage collection during this time,
// but observationally this ends up being enough time. The caller must also ensure that the buffer will go out of scope
// shortly after calling this.
// We have to pass in `callRust` because we can call this function from a variety of runtimes.
// Note that assertions on garbage collection are extremely sensitive to exactly how these functions are written,
// and can easily break if you restucture the function, use a different/newer browser, etc!
export const expectDeallocationOrUnregister = (
  callRust: CallRust,
  zapArray: ZapArray
): Promise<void> => {
  // Deallocation code is only run in WASM for now.
  if (jsRuntime === "cef") return Promise.resolve();

  const buffer = zapArray.buffer as ZapBuffer;
  return buffer.readonly
    ? arcAllocated(callRust, buffer).then((arcPtr) => arcDeallocated(arcPtr))
    : vecDeallocated(buffer.__zaplibBufferData.bufferPtr);
};

export let inTest = false;
// Set this to true to enable testing code
export const setInTest = (v: boolean): void => {
  inTest = v;
};
