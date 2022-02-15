import { RpcMouseEvent, RpcTouchEvent, RpcWheelEvent } from "make_rpc_event";
import {
  TextareaEvent,
  TextareaEventKeyDown,
  TextareaEventKeyUp,
  TextareaEventTextInput,
} from "make_textarea";
import {
  FileHandle,
  MutableBufferData,
  PostMessageTypedArray,
  RustZapParam,
  SizingData,
  TlsAndStackData,
  ZapArray,
} from "types";

// Helpers to provide specific typing for Rpcs.
// RpcSpec is a generic Rpc descriptor where we can send and receive events.
// Each entry of send and receive is modeled as:
// Record<Event Name, [Inputs, Outputs]>
// Below, we create a more specific version of this for each Rpc.
export type RpcSpec = {
  send: Record<string, [unknown, unknown]>;
  receive: Record<string, [unknown, unknown]>;
};
// A flipped version of an Rpc type, so that it can be used on the opposite side and
// we ensure our sends and receives match.
// Usage:
//   on main_thread: const rpc = new Rpc<SomeWorkerRpc>(worker);
//   in worker: const rpc = new Rpc<Worker<SomeWorkerRpc>>(self);
export type Worker<T extends RpcSpec> = {
  receive: T["send"];
  send: T["receive"];
};

export type WorkerCallRustParams = {
  name: string;
  params: (string | PostMessageTypedArray | ZapArray)[];
};

export enum WorkerEvent {
  CallRust = "WorkerEvent.CallRust",
  CreateBuffer = "WorkerEvent.CreateBuffer",
  CreateReadOnlyBuffer = "WorkerEvent.CreateReadOnlyBuffer",
  BindMainWorkerPort = "WorkerEvent.BindMainWorkerPort",
  DecrementArc = "WorkerEvent.DecrementArc",
  DeallocVec = "WorkerEvent.DeallocVec",
  IncrementArc = "WorkerEvent.IncrementArc",
  DragEnter = "WorkerEvent.DragEnter",
  DragOver = "WorkerEvent.DragOver",
  DragLeave = "WorkerEvent.DragLeave",
  Drop = "WorkerEvent.Drop",
  WindowMouseUp = "WorkerEvent.WindowMouseUp",
  CanvasMouseDown = "WorkerEvent.CanvasMouseDown",
  WindowMouseMove = "WorkerEvent.WindowMouseMove",
  WindowMouseOut = "WorkerEvent.WindowMouseOut",
  WindowFocus = "WorkerEvent.WindowFocus",
  WindowBlur = "WorkerEvent.WindowBlur",
  ScreenResize = "WorkerEvent.ScreenResize",
  CanvasWheel = "WorkerEvent.CanvasWheel",
  ShowIncompatibleBrowserNotification = "WorkerEvent.ShowIncompatibleBrowserNotification",
  RemoveLoadingIndicators = "WorkerEvent.RemoveLoadingIndicators",
  SetDocumentTitle = "WorkerEvent.SetDocumentTitle",
  SetMouseCursor = "WorkerEvent.SetMouseCursor",
  Fullscreen = "WorkerEvent.Fullscreen",
  Normalscreen = "WorkerEvent.Normalscreen",
  TextCopyResponse = "WorkerEvent.TextCopyResponse",
  EnableGlobalFileDropTarget = "WorkerEvent.EnableGlobalFileDropTarget",
  CallJs = "WorkerEvent.CallJs",
  ShowTextIME = "WorkerEvent.ShowTextIME",
  TextInput = "WorkerEvent.TextInput",
  TextCopy = "WorkerEvent.TextCopy",
  KeyDown = "WorkerEvent.KeyDown",
  KeyUp = "WorkerEvent.KeyUp",
  Init = "WorkerEvent.Init",
  RunWebGL = "WorkerEvent.RunWebGL",
  ThreadSpawn = "WorkerEvent.ThreadSpawn",
  WindowTouchStart = "WorkerEvent.WindowTouchStart",
  WindowTouchMove = "WorkerEvent.WindowTouchMove",
  WindowTouchEndCancelLeave = "WorkerEvent.WindowTouchEndCancelLeave",
  Panic = "WorkerEvent.Panic",
}
export type WasmWorkerRpc = {
  send: {
    [WorkerEvent.BindMainWorkerPort]: [MessagePort, void];
    [WorkerEvent.DecrementArc]: [number, void];
    [WorkerEvent.DeallocVec]: [MutableBufferData, void];
    [WorkerEvent.IncrementArc]: [number, void];
    [WorkerEvent.CallRust]: [WorkerCallRustParams, Promise<RustZapParam[]>];
    [WorkerEvent.CreateBuffer]: [ZapArray, number];
    [WorkerEvent.CreateReadOnlyBuffer]: [
      ZapArray,
      {
        bufferPtr: number;
        arcPtr: number;
      }
    ];
    [WorkerEvent.DragEnter]: [void, void];
    [WorkerEvent.DragOver]: [{ x: number; y: number }, void];
    [WorkerEvent.DragLeave]: [void, void];
    [WorkerEvent.Drop]: [
      { fileHandles: FileHandle[]; fileHandlesToSend: FileHandle[] },
      void
    ];
    [WorkerEvent.CanvasMouseDown]: [RpcMouseEvent, void];
    [WorkerEvent.WindowMouseUp]: [RpcMouseEvent, void];
    [WorkerEvent.WindowMouseMove]: [RpcMouseEvent, void];
    [WorkerEvent.WindowMouseOut]: [RpcMouseEvent, void];
    [WorkerEvent.WindowTouchStart]: [RpcTouchEvent, void];
    [WorkerEvent.WindowTouchMove]: [RpcTouchEvent, void];
    [WorkerEvent.WindowTouchEndCancelLeave]: [RpcTouchEvent, void];
    [WorkerEvent.CanvasWheel]: [RpcWheelEvent, void];
    [WorkerEvent.WindowFocus]: [RpcWheelEvent, void];
    [WorkerEvent.WindowBlur]: [RpcWheelEvent, void];
    [WorkerEvent.KeyDown]: [TextareaEventKeyDown, void];
    [WorkerEvent.KeyUp]: [TextareaEventKeyUp, void];
    [WorkerEvent.TextInput]: [TextareaEventTextInput, void];
    [WorkerEvent.TextCopy]: [TextareaEvent, void];
    [WorkerEvent.ScreenResize]: [SizingData, void];
    [WorkerEvent.ShowIncompatibleBrowserNotification]: [void, void];
    [WorkerEvent.Init]: [
      {
        wasmModule: WebAssembly.Module;
        offscreenCanvas: OffscreenCanvas | undefined;
        sizingData: SizingData;
        baseUri: string;
        memory: WebAssembly.Memory;
        taskWorkerSab: SharedArrayBuffer;
        tlsAndStackData: TlsAndStackData;
        appPtr: BigInt;
        wasmOnline: Uint8Array;
      },
      void
    ];
  };
  receive: {
    [WorkerEvent.ShowIncompatibleBrowserNotification]: [void, void];
    [WorkerEvent.RemoveLoadingIndicators]: [void, void];
    [WorkerEvent.SetDocumentTitle]: [string, void];
    [WorkerEvent.SetMouseCursor]: [string, void];
    [WorkerEvent.Fullscreen]: [void, void];
    [WorkerEvent.Normalscreen]: [void, void];
    [WorkerEvent.TextCopyResponse]: [string, void];
    [WorkerEvent.EnableGlobalFileDropTarget]: [void, void];
    [WorkerEvent.CallJs]: [
      {
        fnName: string;
        params: RustZapParam[];
      },
      void
    ];
    [WorkerEvent.ShowTextIME]: [{ x: number; y: number }, void];
    [WorkerEvent.RunWebGL]: [number, void];
    [WorkerEvent.ThreadSpawn]: [
      {
        ctxPtr: BigInt;
        tlsAndStackData: TlsAndStackData;
      },
      void
    ];
    [WorkerEvent.Panic]: [Error, void];
  };
};

export enum TaskWorkerEvent {
  Init = "TaskWorkerEvent.Init",
}
export type TaskWorkerRpc = {
  send: {
    [TaskWorkerEvent.Init]: [
      {
        taskWorkerSab: SharedArrayBuffer;
        wasmMemory: WebAssembly.Memory;
      },
      void
    ];
  };
  receive: Record<string, never>;
};

export enum AsyncWorkerEvent {
  Run = "AsyncWorkerEvent.Run",
  ThreadSpawn = "AsyncWorkerEvent.ThreadSpawn",
}
export type AsyncWorkerRpc = {
  send: {
    [AsyncWorkerEvent.Run]: [
      {
        wasmModule: WebAssembly.Module;
        memory: WebAssembly.Memory;
        taskWorkerSab: SharedArrayBuffer;
        ctxPtr: BigInt;
        fileHandles: FileHandle[];
        baseUri: string;
        tlsAndStackData: TlsAndStackData;
        mainWorkerPort: MessagePort;
      },
      void
    ];
  };
  receive: {
    [AsyncWorkerEvent.ThreadSpawn]: [
      {
        ctxPtr: BigInt;
        tlsAndStackData: TlsAndStackData;
      },
      void
    ];
  };
};

export enum MainWorkerChannelEvent {
  Init = "MainWorkerChannelEvent.Init",
  BindMainWorkerPort = "MainWorkerChannelEvent.BindMainWorkerPort",
  CallRust = "MainWorkerChannelEvent.CallRust",
  SendEventFromAnyThread = "MainWorkerChannelEvent.SendEventFromAnyThread",
}
export type WebWorkerRpc = {
  send: {
    [MainWorkerChannelEvent.BindMainWorkerPort]: [MessagePort, void];
    [MainWorkerChannelEvent.Init]: [
      void,
      {
        wasmModule: WebAssembly.Module;
        memory: WebAssembly.Memory;
        taskWorkerSab: SharedArrayBuffer;
        appPtr: BigInt;
        baseUri: string;
        tlsAndStackData: TlsAndStackData;
        wasmOnline: Uint8Array;
      }
    ];
    [MainWorkerChannelEvent.SendEventFromAnyThread]: [BigInt, void];
    [MainWorkerChannelEvent.CallRust]: [
      WorkerCallRustParams,
      Promise<RustZapParam[]>
    ];
  };
  receive: Record<string, never>;
};
