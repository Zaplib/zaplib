import {
  Rpc,
  TW_SAB_MUTEX_PTR,
  TW_SAB_MESSAGE_COUNT_PTR,
  mutexLock,
  mutexUnlock,
  assertNotNull,
} from "./common";
import { Worker, TaskWorkerRpc, TaskWorkerEvent } from "./rpc_types";
import { ZerdeParser } from "./zerde";

/// <reference lib="WebWorker" />

// This "task worker" is a special worker that helps convert asynchronous JavaScript APIs to synchronous,
// blocking APIs.
//
// Consider the following example: (TODO(JP): this particular example is not implemented yet but it's the easiest example..)
//
//   let handler = thread::spawn(|| {
//     // thread code
//   });
//   handler.join(); // blocks until the thread is done
//
// In a naive implementation, `thread::spawn` would simply call `new Worker` in JavaScript. However,
// the `handler.join()` call makes the thread block. This prevents JavaScript from actually creating
// the worker, because `new Worker` is an asynchronous API which only actually spawns a new worker when
// control is given back to the JavaScript event loop. So this would block forever!
//
// We solve this by having this task worker wait for messages on a `SharedArrayBuffer`, using
// `Atomics.wait`. This way instead of calling `new Worker` in the other thread, we can append a
// message, and call `Atomics.notify`. This wakes up the task worker and calls `new Worker`.
//
// Now, consider a more complicated example:
//
//  let reader = request("url").unwrap(); // blocks until we have a valid HTTP connection
//
// This would map to a `fetch` call in JavaScript. However, that returns a Promise, and there is no way
// to directly block on that Promise! Again, we can use the task worker to help, although the full flow
// is quite a bit more complicated:
//
// * Task worker is waiting for a message on its `SharedArrayBuffer`, using `Atomics.wait`.
// * User thread appends a message (containing e.g. the URL, and a pointer on the main `memory` to get
//   a notification when the task worker is done) and notifies the task worker using `Atomics.notify`.
// * User thread uses `Atomics.wait` to wait for information back from the task worker.
// * Task worker gets woken up, parses the messages, calls `fetch`, and increments `async_tasks`.
// * Task worker doesn't use `Atomics.wait` to block, but instead it relinquishes control to JavaScript,
//   so that the JavaScript event loop can run, and call the Promise callback when done.
// * In order to not miss any other messages (from other threads), the task worker uses `setTimeout`
//   to poll for new messages.
// * The JavaScript event loop calls the Promise callback, which decrements `async_tasks`, and returns
//   information back to the user thread using the pointer that was supplied in the original message,
//   and calls Atomics.notify to wake up the user thread.
// * If there aren't any other in-flight tasks (`async_tasks` is 0), the task worker blocks again to
//   wait for the next message, using `Atomics.wait`.
//
// See `initTaskWorkerSab` and `sendTaskWorkerMessage` in `common.js` for how we
// communicate messages to this worker.

type TaskWorkerMessage = {
  bytesReadReturnValPtr: number;
  streamId: number;
  bufPtr: number;
  bufLen: number;
};

const _TASK_WORKER_INITIAL_RETURN_VALUE = -1;
const TASK_WORKER_ERROR_RETURN_VALUE = -2;

const rpc = new Rpc<Worker<TaskWorkerRpc>>(self);
rpc.receive(TaskWorkerEvent.Init, ({ taskWorkerSab, wasmMemory }) => {
  const taskWorkerSabi32 = new Int32Array(taskWorkerSab);

  // Number of async tasks that require the JavaScript even loop to have control. If zero, we'll
  // use Atomics.wait to wait for the next message, otherwise we'll use setTimeout to poll for new
  // messages.
  let asyncTasks = 0;

  // HTTP streams. Start IDs with 1, since 0 signifies an error.
  let nextStreamId = 1;
  const streams: Record<
    number,
    {
      reader: ReadableStreamDefaultReader<any>;
      done: boolean;
      values: Uint8Array[];
      error: boolean;
      currentTwMessage: TaskWorkerMessage | undefined;
    }
  > = {};

  // Send back an i32 return value, and wake up the original thread.
  function sendi32ReturnValue(returnValPtr: number, returnValue: number) {
    const memoryReturni32 = new Int32Array(wasmMemory.buffer, returnValPtr, 1);
    if (memoryReturni32[0] === returnValue) {
      throw new Error(
        "Have to set the return value to something different than the initial value, otherwise Atomics.notify won't do anything"
      );
    }
    memoryReturni32[0] = returnValue;
    Atomics.notify(memoryReturni32, 0);
  }

  // Make a new read request for a given stream. We do this even if the underlying application doesn't
  // ask for it, so that we can return bytes in the fastest manner possible.
  // TODO(JP): We might want to set a limit to how much we buffer ahead? Or make it configurable per stream?
  function readDataIntoValuesBuffer(streamId: number) {
    const stream = streams[streamId];
    asyncTasks++;
    stream.reader
      .read()
      .then((readResponse) => {
        asyncTasks--;
        if (readResponse.done) {
          stream.done = true;
        } else {
          stream.values.push(readResponse.value);
          readDataIntoValuesBuffer(streamId);
        }
        handleHttpStreamRead(streamId);
      })
      .catch((error) => {
        asyncTasks--;
        // TODO(JP): Actually return the error to Rust at some point. For now we just print it.
        console.error("fetch read error", error);
        stream.error = true;
        handleHttpStreamRead(streamId);
      });
  }

  // Check if we can supply a "read" call with data. There are two cases in which this can happen:
  // * There is a new read call, and there is a sufficient amount of data to give it.
  // * There is new data, and there is an existing read call to hand it to.
  // In other cases we buffer the data or block the read call, and wait until we have enough of both.
  function handleHttpStreamRead(streamId: number) {
    const stream = streams[streamId];
    if (!stream.currentTwMessage) {
      // If there isn't a read call we can satisfy, bail.
      return;
    }

    if (stream.error) {
      sendi32ReturnValue(
        stream.currentTwMessage.bytesReadReturnValPtr,
        TASK_WORKER_ERROR_RETURN_VALUE
      );
      stream.currentTwMessage = undefined;
      return;
    }

    if (stream.values.length === 0) {
      if (stream.done) {
        // If there isn't more data, and we've reached the end of the stream, just return that we read 0 bytes.
        sendi32ReturnValue(stream.currentTwMessage.bytesReadReturnValPtr, 0);
        stream.currentTwMessage = undefined;
      }
      // If there is no more data but we're not done yet, just bail.
      return;
    }

    // Read as many bytes as we can stuff in the buffer that was supplied to us from the read call.
    let bytesRead = 0;
    while (
      stream.values.length > 0 &&
      bytesRead < stream.currentTwMessage.bufLen
    ) {
      const value = stream.values[0];

      const remainingBytesToRead = stream.currentTwMessage.bufLen - bytesRead;
      const bytesToReadFromValue = Math.min(
        value.byteLength,
        remainingBytesToRead
      );

      const sourceBuffer = new Uint8Array(
        value.buffer,
        value.byteOffset,
        bytesToReadFromValue
      );
      new Uint8Array(
        wasmMemory.buffer,
        stream.currentTwMessage.bufPtr + bytesRead,
        bytesToReadFromValue
      ).set(sourceBuffer);

      if (bytesToReadFromValue < value.byteLength) {
        // If we weren't able to read the entire buffer, replace it with a buffer containing the rest.
        stream.values[0] = new Uint8Array(
          value.buffer,
          value.byteOffset + bytesToReadFromValue,
          value.byteLength - bytesToReadFromValue
        );
      } else {
        // If we read the whole buffer, remove it.
        stream.values.shift();
      }

      bytesRead += bytesToReadFromValue;
    }

    // Return the number of bytes that we read.
    sendi32ReturnValue(
      stream.currentTwMessage.bytesReadReturnValPtr,
      bytesRead
    );
    stream.currentTwMessage = undefined;
  }

  // Parse a message, which is formatted using `ZerdeBuilder` in Rust, so we use `ZerdeParser` in JavaScript
  // to decode it.
  function handleTwMessage(zerdeParser: ZerdeParser) {
    const messageType = zerdeParser.parseU32();

    if (messageType == 1) {
      // http_stream_new
      const streamIdReturnValPtr = zerdeParser.parseU32();
      const url = zerdeParser.parseString();
      const method = zerdeParser.parseString();
      const body = zerdeParser.parseU8Slice();
      const numberOfHeaders = zerdeParser.parseU32();
      const headers: Record<string, string> = {};
      for (let headerIndex = 0; headerIndex < numberOfHeaders; headerIndex++) {
        headers[zerdeParser.parseString()] = zerdeParser.parseString();
      }

      asyncTasks++;
      fetch(url, { method, body, headers })
        .then((response) => {
          asyncTasks--;

          if (response.ok) {
            const streamId = nextStreamId++;
            streams[streamId] = {
              // An asynchronous reader, which returns "chunks"/"values" of data.
              // TODO(JP): Switch to "byob" when that's supported here; see
              // https://bugs.chromium.org/p/chromium/issues/detail?id=614302#c23
              reader: assertNotNull(response.body).getReader(),
              // The buffered "chunks"/"values".
              values: [],
              // Whether we've read the whole stream into `values`.
              done: false,
              // Whether we encountered an error during reading.
              error: false,
              // The current read message to return data for, if any.
              currentTwMessage: undefined,
            };
            readDataIntoValuesBuffer(streamId);
            sendi32ReturnValue(streamIdReturnValPtr, streamId);
          } else {
            // TODO(JP): Actually return the status code to Rust at some point. For now you'll just
            // have to look at the Network tab of the browser's developer tools.
            sendi32ReturnValue(
              streamIdReturnValPtr,
              TASK_WORKER_ERROR_RETURN_VALUE
            );
          }
        })
        .catch((error) => {
          asyncTasks--;
          // TODO(JP): Actually return the error to Rust at some point. For now we just print it.
          console.error("fetch create error", error);
          sendi32ReturnValue(
            streamIdReturnValPtr,
            TASK_WORKER_ERROR_RETURN_VALUE
          );
        });
    } else if (messageType == 2) {
      // http_stream_read
      const twMessage: TaskWorkerMessage = {
        bytesReadReturnValPtr: zerdeParser.parseU32(),
        streamId: zerdeParser.parseU32(),
        bufPtr: zerdeParser.parseU32(),
        bufLen: zerdeParser.parseU32(),
      };
      if (streams[twMessage.streamId].currentTwMessage) {
        // TODO(JP): Actually return the error to Rust at some point. For now we just print it.
        console.error("Got multiple http_stream_read messages in a row");
        sendi32ReturnValue(
          twMessage.bytesReadReturnValPtr,
          TASK_WORKER_ERROR_RETURN_VALUE
        );
        return;
      }
      streams[twMessage.streamId].currentTwMessage = twMessage;
      handleHttpStreamRead(twMessage.streamId);
    }
  }

  function process() {
    // eslint-disable-next-line no-constant-condition
    while (true) {
      // Check if there are any messages. We do this without setting the Mutex, since
      // assume that reads are always safe. Worse case we read an incorrect value, but
      // a few lines down we read it again after having the Mutex.
      if (Atomics.load(taskWorkerSabi32, TW_SAB_MESSAGE_COUNT_PTR) > 0) {
        mutexLock(taskWorkerSabi32, TW_SAB_MUTEX_PTR);
        // Read the number of messages again now that we have the Mutex.
        const numberOfMessages = taskWorkerSabi32[1];

        // Handle all messages.
        for (
          let messageIndex = 0;
          messageIndex < numberOfMessages;
          messageIndex++
        ) {
          // Use unsigned numbers for the actual pointer, since they can be >2GB.
          const messagePtr = new Uint32Array(taskWorkerSab)[messageIndex + 2];
          handleTwMessage(new ZerdeParser(wasmMemory, messagePtr));
        }

        // Reset the number of messages to 0.
        taskWorkerSabi32[TW_SAB_MESSAGE_COUNT_PTR] = 0;
        mutexUnlock(taskWorkerSabi32, TW_SAB_MUTEX_PTR);
      }

      if (asyncTasks > 0) {
        // We can't block if we have any async tasks currently running, since we need
        // the JavaScript event loop to be in control. So we queue up a new call to
        // this function (which will be handled by the event loop!) and bail.
        setTimeout(process, 1);
        break;
      } else {
        // Otherwise, we can safely block to wait for the next message.
        Atomics.wait(taskWorkerSabi32, 1, 0);
      }
    }
  }
  // Queue up the first call to `process`. Don't call it directly, because it will likely immediately block,
  // and it's nice to resolve the Promise associated with this "init" call (even though currently we don't
  // actually use it).
  setTimeout(process, 0);
});
