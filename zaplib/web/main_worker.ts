import { cursorMap } from "cursor_map";
import {
  Rpc,
  getWasmEnv,
  makeThreadLocalStorageAndStackDataOnExistingThread,
  createErrorCheckers,
  initThreadLocalStorageAndStackOtherWorkers,
} from "common";
import {
  TextareaEventKeyDown,
  TextareaEventKeyUp,
  TextareaEventTextInput,
} from "make_textarea";
import {
  FileHandle,
  WasmExports,
  SizingData,
  ZapArray,
  MutableBufferData,
  RustZapParam,
} from "types";
import { ZerdeParser } from "zerde";
import { ZerdeEventloopEvents } from "zerde_eventloop_events";
import { packKeyModifier } from "zerde_keyboard_handlers";
import { WebGLRenderer } from "webgl_renderer";
import { RpcMouseEvent, RpcTouchEvent, RpcWheelEvent } from "make_rpc_event";
import {
  Worker,
  WasmWorkerRpc,
  WebWorkerRpc,
  WorkerCallRustAsyncParams,
  WorkerEvent,
  MainWorkerChannelEvent,
} from "rpc_types";

const rpc = new Rpc<Worker<WasmWorkerRpc>>(globalThis);

const isFirefox =
  globalThis.navigator?.userAgent.toLowerCase().indexOf("firefox") > -1;
// var is_add_to_homescreen_safari = is_mobile_safari && navigator.standalone;
//var is_oculus_browser = navigator.userAgent.indexOf('OculusBrowser') > -1;

type Timer = { id: number; repeats: number; sysId: NodeJS.Timer };

export type Pointer = {
  x: number;
  y: number;
  button: number;
  digit: number;
  time: number;
  modifiers: number;
  touch: boolean;
};

export type PointerScroll = Pointer & {
  scrollX: number;
  scrollY: number;
  isWheel: boolean;
};

// TODO(Paras): Stop patching sendStack onto websockets
// and maintain our own structure instead.
type WebSocketWithSendStack = WebSocket & {
  sendStack?: Uint8Array[] | null;
};

let wasmOnline: Uint8Array;
const wasmInitialized = () => Atomics.load(wasmOnline, 0) === 1;
const { wrapWasmExports } = createErrorCheckers(wasmInitialized);

export class WasmApp {
  memory: WebAssembly.Memory;
  exports: WasmExports;
  module: WebAssembly.Module;
  private sizingData: SizingData;
  private baseUri: string;
  private timers: Timer[];
  private hasRequestedAnimationFrame: boolean;
  private websockets: Record<string, WebSocketWithSendStack | null>;
  private fileHandles: FileHandle[];
  private zerdeEventloopEvents: ZerdeEventloopEvents;
  private appPtr: BigInt;
  private doWasmBlock!: boolean;
  private xrCanPresent = false;
  private xrIsPresenting = false;
  private zerdeParser!: ZerdeParser;
  private callRustAsyncNewCallbackId: number;
  private callRustAsyncPendingCallbacks: Record<
    number,
    (arg0: RustZapParam[]) => void
  >;
  // WebGLRenderer if we're using an OffscreenCanvas. If not, this is undefined.
  private webglRenderer: WebGLRenderer | undefined;
  // Promise which is set when we have an active RunWebGL call in the main browser thread.
  private runWebGLPromise: Promise<void> | undefined;

  constructor({
    offscreenCanvas,
    wasmModule,
    wasmExports,
    memory,
    sizingData,
    baseUri,
    fileHandles,
    taskWorkerSab,
    appPtr,
  }: {
    offscreenCanvas: OffscreenCanvas | undefined;
    wasmModule: WebAssembly.Module;
    wasmExports: WasmExports;
    memory: WebAssembly.Memory;
    sizingData: SizingData;
    baseUri: string;
    fileHandles: FileHandle[];
    taskWorkerSab: SharedArrayBuffer;
    appPtr: BigInt;
  }) {
    this.module = wasmModule;
    this.exports = wasmExports;
    this.memory = memory;
    this.baseUri = baseUri;
    this.sizingData = sizingData;
    this.appPtr = appPtr;

    this.timers = [];
    this.hasRequestedAnimationFrame = false;
    this.websockets = {};
    this.fileHandles = fileHandles;

    this.callRustAsyncNewCallbackId = 0;
    this.callRustAsyncPendingCallbacks = {};

    if (offscreenCanvas) {
      this.webglRenderer = new WebGLRenderer(
        offscreenCanvas,
        this.memory,
        this.sizingData,
        () => {
          rpc.send(WorkerEvent.ShowIncompatibleBrowserNotification);
        }
      );
    }

    rpc.receive(WorkerEvent.ScreenResize, (sizingData: SizingData) => {
      this.sizingData = sizingData;
      if (this.webglRenderer) {
        this.webglRenderer.resize(this.sizingData);
      }

      this.zerdeEventloopEvents.resize({
        width: this.sizingData.width,
        height: this.sizingData.height,
        dpiFactor: this.sizingData.dpiFactor,
        xrIsPresenting: this.xrIsPresenting,
        xrCanPresent: this.xrCanPresent,
        isFullscreen: this.sizingData.isFullscreen,
      });
      this.requestAnimationFrame();
    });

    // this.run_async_webxr_check();
    this.bindMouseAndTouch();
    this.bindKeyboard();

    rpc.receive(WorkerEvent.WindowFocus, () => {
      this.zerdeEventloopEvents.windowFocus(true);
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.WindowBlur, () => {
      this.zerdeEventloopEvents.windowFocus(false);
      this.doWasmIo();
    });

    const callRustAsync = ({
      name,
      params,
    }: WorkerCallRustAsyncParams): Promise<RustZapParam[]> => {
      const callbackId = this.callRustAsyncNewCallbackId++;
      const promise = new Promise<RustZapParam[]>((resolve, _reject) => {
        this.callRustAsyncPendingCallbacks[callbackId] = (
          data: RustZapParam[]
        ) => {
          // TODO(Dmitry): implement retrun_error on rust side and use reject(...) to communicate the error
          resolve(data);
        };
      });

      this.zerdeEventloopEvents.callRustAsync(name, params, callbackId);
      this.doWasmIo();
      return promise;
    };
    rpc.receive(WorkerEvent.CallRustAsync, callRustAsync);

    rpc.receive(WorkerEvent.CreateBuffer, (data: ZapArray) =>
      this.zerdeEventloopEvents.createWasmBuffer(data)
    );

    rpc.receive(WorkerEvent.CreateReadOnlyBuffer, (data: ZapArray) => {
      const bufferPtr = this.zerdeEventloopEvents.createWasmBuffer(data);
      const arcPtr = this.zerdeEventloopEvents.createArcVec(bufferPtr, data);
      return { bufferPtr, arcPtr };
    });

    rpc.receive(WorkerEvent.IncrementArc, (arcPtr: number) => {
      this.exports.incrementArc(BigInt(arcPtr));
    });

    rpc.receive(WorkerEvent.DecrementArc, (arcPtr: number) => {
      this.exports.decrementArc(BigInt(arcPtr));
    });

    rpc.receive(
      WorkerEvent.DeallocVec,
      ({ bufferPtr, bufferLen, bufferCap }: MutableBufferData) => {
        this.exports.deallocVec(
          BigInt(bufferPtr),
          BigInt(bufferLen),
          BigInt(bufferCap)
        );
      }
    );

    const bindMainWorkerPort = (port: MessagePort) => {
      const userWorkerRpc = new Rpc<Worker<WebWorkerRpc>>(port);
      userWorkerRpc.receive(MainWorkerChannelEvent.Init, () => ({
        wasmModule: this.module,
        memory: this.memory,
        taskWorkerSab,
        appPtr: this.appPtr,
        baseUri,
        tlsAndStackData: makeThreadLocalStorageAndStackDataOnExistingThread(
          this.exports
        ),
        wasmOnline,
      }));
      userWorkerRpc.receive(
        MainWorkerChannelEvent.BindMainWorkerPort,
        (port: MessagePort) => {
          bindMainWorkerPort(port);
        }
      );

      userWorkerRpc.receive(
        MainWorkerChannelEvent.CallRustAsync,
        callRustAsync
      );

      userWorkerRpc.receive(
        MainWorkerChannelEvent.SendEventFromAnyThread,
        (eventPtr: BigInt) => {
          this.sendEventFromAnyThread(eventPtr);
        }
      );
    };
    rpc.receive(WorkerEvent.BindMainWorkerPort, (port) => {
      bindMainWorkerPort(port);
    });

    // create initial zerdeEventloopEvents
    this.zerdeEventloopEvents = new ZerdeEventloopEvents(this);
  }

  // This is separate from the constructor, since this can cause calls
  // to callbacks in `getWasmEnv`, which refer to `wasmapp`, so we need
  // the constructor to have finished.
  init(): void {
    Atomics.store(wasmOnline, 0, 1);
    this.exports = wrapWasmExports(this.exports);

    rpc.send(WorkerEvent.RemoveLoadingIndicators);

    // initialize the application
    this.zerdeEventloopEvents.init({
      width: this.sizingData.width,
      height: this.sizingData.height,
      dpiFactor: this.sizingData.dpiFactor,
      xrCanPresent: this.xrCanPresent,
      canFullscreen: this.sizingData.canFullscreen,
      xrIsPresenting: false,
    });
    this.doWasmIo();
  }

  private doWasmIo(): void {
    if (this.doWasmBlock) {
      return;
    }

    const byteOffset = this.zerdeEventloopEvents.end();
    const zerdeParserPtr = Number(
      this.exports.processWasmEvents(this.appPtr, BigInt(byteOffset))
    );

    // get a clean zerdeEventloopEvents set up immediately
    this.zerdeEventloopEvents = new ZerdeEventloopEvents(this);
    this.zerdeParser = new ZerdeParser(this.memory, zerdeParserPtr);

    // eslint-disable-next-line no-constant-condition
    while (true) {
      const msgType = this.zerdeParser.parseU32();
      if (this.sendFnTable[msgType](this)) {
        break;
      }
    }

    this.exports.deallocWasmMessage(BigInt(zerdeParserPtr));
  }

  private setDocumentTitle(title: string): void {
    rpc.send(WorkerEvent.SetDocumentTitle, title);
  }

  private bindMouseAndTouch(): void {
    let lastMousePointer;
    // TODO(JP): Some day bring back touch scroll support..
    // let use_touch_scroll_overlay = window.ontouchstart === null;
    // if (use_touch_scroll_overlay) {
    //     var ts = this.touch_scroll_overlay = document.createElement('div')
    //     ts.className = "cx_webgl_scroll_overlay"
    //     var ts_inner = document.createElement('div')
    //     var style = document.createElement('style')
    //     style.innerHTML = "\n"
    //         + "div.cx_webgl_scroll_overlay {\n"
    //         + "z-index: 10000;\n"
    //         + "margin:0;\n"
    //         + "overflow:scroll;\n"
    //         + "top:0;\n"
    //         + "left:0;\n"
    //         + "width:100%;\n"
    //         + "height:100%;\n"
    //         + "position:fixed;\n"
    //         + "background-color:transparent\n"
    //         + "}\n"
    //         + "div.cx_webgl_scroll_overlay div{\n"
    //         + "margin:0;\n"
    //         + "width:400000px;\n"
    //         + "height:400000px;\n"
    //         + "background-color:transparent\n"
    //         + "}\n"

    //     document.body.appendChild(style)
    //     ts.appendChild(ts_inner);
    //     document.body.appendChild(ts);
    //     canvas = ts;

    //     ts.scrollTop = 200000;
    //     ts.scrollLeft = 200000;
    //     let last_scroll_top = ts.scrollTop;
    //     let last_scroll_left = ts.scrollLeft;
    //     let scroll_timeout = null;
    //     ts.addEventListener('scroll', e => {
    //         let new_scroll_top = ts.scrollTop;
    //         let new_scroll_left = ts.scrollLeft;
    //         let dx = new_scroll_left - last_scroll_left;
    //         let dy = new_scroll_top - last_scroll_top;
    //         last_scroll_top = new_scroll_top;
    //         last_scroll_left = new_scroll_left;
    //         globalThis.clearTimeout(scroll_timeout);
    //         scroll_timeout = globalThis.setTimeout(_ => {
    //             ts.scrollTop = 200000;
    //             ts.scrollLeft = 200000;
    //             last_scroll_top = ts.scrollTop;
    //             last_scroll_left = ts.scrollLeft;
    //         }, 200);

    //         let pointer = last_mouse_pointer;
    //         if (pointer) {
    //             pointer.scroll_x = dx;
    //             pointer.scroll_y = dy;
    //             pointer.is_wheel = true;
    //             this.zerdeEventloopEvents.pointer_scroll(pointer);
    //             this.do_wasm_io();
    //         }
    //     })
    // }

    const mousePointers: {
      x: number;
      y: number;
      button: number;
      digit: number;
      time: number;
      modifiers: number;
      touch: boolean;
    }[] = [];
    function mouseToPointer(e: RpcMouseEvent | RpcWheelEvent): Pointer {
      // @ts-ignore; TypeScript does not like the empty object declaration below, but we immediately fill every field
      const mf = mousePointers[e.button] || (mousePointers[e.button] = {});
      mf.x = e.pageX;
      mf.y = e.pageY;
      mf.button = e.button;
      mf.digit = e.button;
      mf.time = performance.now() / 1000.0;
      mf.modifiers = packKeyModifier(e);
      mf.touch = false;
      return mf;
    }

    const mouseButtonsDown: boolean[] = [];
    rpc.receive(WorkerEvent.CanvasMouseDown, (event: RpcMouseEvent) => {
      mouseButtonsDown[event.button] = true;
      this.zerdeEventloopEvents.pointerDown(mouseToPointer(event));
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.WindowMouseUp, (event: RpcMouseEvent) => {
      mouseButtonsDown[event.button] = false;
      this.zerdeEventloopEvents.pointerUp(mouseToPointer(event));
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.WindowMouseMove, (event: RpcMouseEvent) => {
      for (let i = 0; i < mouseButtonsDown.length; i++) {
        if (mouseButtonsDown[i]) {
          const mf = mouseToPointer(event);
          mf.digit = i;
          this.zerdeEventloopEvents.pointerMove(mf);
        }
      }
      lastMousePointer = mouseToPointer(event);
      this.zerdeEventloopEvents.pointerHover(lastMousePointer);
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.WindowMouseOut, (event: RpcMouseEvent) => {
      this.zerdeEventloopEvents.pointerOut(mouseToPointer(event));
      this.doWasmIo();
    });

    const touchIdsByDigit: (number | undefined)[] = [];
    rpc.receive(WorkerEvent.WindowTouchStart, (event: RpcTouchEvent) => {
      for (const touch of event.changedTouches) {
        let digit = touchIdsByDigit.indexOf(undefined);
        if (digit === -1) {
          digit = touchIdsByDigit.length;
        }
        touchIdsByDigit[digit] = touch.identifier;

        this.zerdeEventloopEvents.pointerDown({
          x: touch.pageX,
          y: touch.pageY,
          button: 0,
          digit,
          time: performance.now() / 1000.0,
          modifiers: packKeyModifier(event),
          touch: true,
        });
      }
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.WindowTouchMove, (event: RpcTouchEvent) => {
      for (const touch of event.changedTouches) {
        const digit = touchIdsByDigit.indexOf(touch.identifier);
        if (digit == -1) {
          console.error("Unrecognized digit in WorkerEvent.WindowTouchMove");
          continue;
        }
        this.zerdeEventloopEvents.pointerMove({
          x: touch.pageX,
          y: touch.pageY,
          button: 0,
          digit,
          time: performance.now() / 1000.0,
          modifiers: packKeyModifier(event),
          touch: true,
        });
      }
      this.doWasmIo();
    });
    rpc.receive(
      WorkerEvent.WindowTouchEndCancelLeave,
      (event: RpcTouchEvent) => {
        for (const touch of event.changedTouches) {
          const digit = touchIdsByDigit.indexOf(touch.identifier);
          if (digit == -1) {
            console.error("Unrecognized digit in WorkerEvent.WindowTouchMove");
            continue;
          }
          touchIdsByDigit[digit] = undefined;
          this.zerdeEventloopEvents.pointerUp({
            x: touch.pageX,
            y: touch.pageY,
            button: 0,
            digit,
            time: performance.now() / 1000.0,
            modifiers: packKeyModifier(event),
            touch: true,
          });
        }
        this.doWasmIo();
      }
    );

    let lastWheelTime: number;
    let lastWasWheel: boolean;
    rpc.receive(WorkerEvent.CanvasWheel, (event: RpcWheelEvent) => {
      const pointer = mouseToPointer(event);
      const delta = event.timeStamp - lastWheelTime;
      lastWheelTime = event.timeStamp;
      // typical web bullshit. this reliably detects mousewheel or touchpad on mac in safari
      if (isFirefox) {
        lastWasWheel = event.deltaMode == 1;
      } else {
        // detect it
        if (
          // @ts-ignore: TODO(Paras): wheelDeltaY looks different between browsers. Figure out a more consistent interface.
          Math.abs(Math.abs(event.deltaY / event.wheelDeltaY) - 1 / 3) <
            0.00001 ||
          (!lastWasWheel && delta < 250)
        ) {
          lastWasWheel = false;
        } else {
          lastWasWheel = true;
        }
      }
      //console.log(event.deltaY / event.wheelDeltaY);
      //last_delta = delta;
      let fac = 1;
      if (event.deltaMode === 1) {
        fac = 40;
      } else if (event.deltaMode === 2) {
        // TODO(Paras): deltaMode=2 means that a user is trying to scroll one page at a time.
        // For now, we hardcode the pixel amount. We can later determine this contextually.
        const offsetHeight = 800;
        fac = offsetHeight;
      }
      const pointerScroll = {
        ...pointer,
        scrollX: event.deltaX * fac,
        scrollY: event.deltaY * fac,
        isWheel: lastWasWheel,
      };
      this.zerdeEventloopEvents.pointerScroll(pointerScroll);
      this.doWasmIo();
    });

    //window.addEventListener('webkitmouseforcewillbegin', this.onCheckMacForce.bind(this), false)
    //window.addEventListener('webkitmouseforcechanged', this.onCheckMacForce.bind(this), false)
  }

  private bindKeyboard(): void {
    rpc.receive(WorkerEvent.TextInput, (data: TextareaEventTextInput) => {
      this.zerdeEventloopEvents.textInput(data);
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.TextCopy, () => {
      this.zerdeEventloopEvents.textCopy();
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.KeyDown, (data: TextareaEventKeyDown) => {
      this.zerdeEventloopEvents.keyDown(data);
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.KeyUp, (data: TextareaEventKeyUp) => {
      this.zerdeEventloopEvents.keyUp(data);
      this.doWasmIo();
    });
  }

  private setMouseCursor(id: number): void {
    rpc.send(WorkerEvent.SetMouseCursor, cursorMap[id] || "default");
  }

  private startTimer(id: number, interval: number, repeats: number): void {
    for (let i = 0; i < this.timers.length; i++) {
      if (this.timers[i].id == id) {
        console.log("Timer ID collision!");
        return;
      }
    }
    const sysId =
      repeats !== 0
        ? globalThis.setInterval(() => {
            this.zerdeEventloopEvents.timerFired(id);
            this.doWasmIo();
          }, interval * 1000.0)
        : globalThis.setTimeout(() => {
            for (let i = 0; i < this.timers.length; i++) {
              const timer = this.timers[i];
              if (timer.id == id) {
                this.timers.splice(i, 1);
                break;
              }
            }
            this.zerdeEventloopEvents.timerFired(id);
            this.doWasmIo();
          }, interval * 1000.0);

    this.timers.push({ id, repeats, sysId });
  }

  private stopTimer(id: number): void {
    for (let i = 0; i < this.timers.length; i++) {
      const timer = this.timers[i];
      if (timer.id == id) {
        if (timer.repeats) {
          globalThis.clearInterval(timer.sysId);
        } else {
          globalThis.clearTimeout(timer.sysId);
        }
        this.timers.splice(i, 1);
        return;
      }
    }
    //console.log("Timer ID not found!")
  }

  private httpSend(
    verb: string,
    path: string,
    proto: string,
    domain: string,
    port: number,
    contentType: string,
    body: Uint8Array,
    signalId: number
  ): void {
    const req = new XMLHttpRequest();
    req.addEventListener("error", (_) => {
      // signal fail
      this.zerdeEventloopEvents.httpSendResponse(signalId, 2);
      this.doWasmIo();
    });
    req.addEventListener("load", (_) => {
      if (req.status !== 200) {
        // signal fail
        this.zerdeEventloopEvents.httpSendResponse(signalId, 2);
      } else {
        //signal success
        this.zerdeEventloopEvents.httpSendResponse(signalId, 1);
      }
      this.doWasmIo();
    });
    req.open(verb, proto + "://" + domain + ":" + port + path, true);
    console.log(verb, proto + "://" + domain + ":" + port + path, body);
    req.send(body.buffer);
  }

  private websocketSend(url: string, data: Uint8Array): void {
    // TODO(Paras): Stop patching sendStack onto websockets
    // and maintain our own structure instead.
    const socket = this.websockets[url];
    if (!socket) {
      const socket = new WebSocket(url) as WebSocketWithSendStack;
      this.websockets[url] = socket;
      socket.sendStack = [data];
      socket.addEventListener("close", () => {
        this.websockets[url] = null;
      });
      socket.addEventListener("error", (event) => {
        this.websockets[url] = null;
        this.zerdeEventloopEvents.websocketError(url, "" + event);
        this.doWasmIo();
      });
      socket.addEventListener("message", (event) => {
        event.data.arrayBuffer().then((data: ArrayBuffer) => {
          this.zerdeEventloopEvents.websocketMessage(url, data);
          this.doWasmIo();
        });
      });
      socket.addEventListener("open", () => {
        const sendStack = socket.sendStack as Uint8Array[];
        socket.sendStack = null;
        for (data of sendStack) {
          socket.send(data);
        }
      });
    } else {
      if (socket.sendStack) {
        socket.sendStack.push(data);
      } else {
        socket.send(data);
      }
    }
  }

  private enableGlobalFileDropTarget(): void {
    rpc.send(WorkerEvent.EnableGlobalFileDropTarget);
    rpc.receive(WorkerEvent.DragEnter, () => {
      this.zerdeEventloopEvents.dragenter();
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.DragOver, ({ x, y }: { x: number; y: number }) => {
      this.zerdeEventloopEvents.dragover(x, y);
      this.doWasmIo();
    });
    rpc.receive(WorkerEvent.DragLeave, () => {
      this.zerdeEventloopEvents.dragleave();
      this.doWasmIo();
    });
    rpc.receive(
      WorkerEvent.Drop,
      ({
        fileHandles,
        fileHandlesToSend,
      }: {
        fileHandles: FileHandle[];
        fileHandlesToSend: FileHandle[];
      }) => {
        // We can't set this.fileHandles to a new object, since other places hold
        // references to it. Instead, clear it out and fill it up again.
        this.fileHandles.splice(0, this.fileHandles.length);
        this.fileHandles.push(...fileHandles);
        this.zerdeEventloopEvents.appOpenFiles(fileHandlesToSend);
        this.doWasmIo();
      }
    );
  }

  private async requestAnimationFrame(): Promise<void> {
    if (this.xrIsPresenting || this.hasRequestedAnimationFrame) {
      return;
    }
    this.hasRequestedAnimationFrame = true;
    if (this.runWebGLPromise) {
      await this.runWebGLPromise;
    }
    (globalThis.requestAnimationFrame || globalThis.setTimeout)(async () => {
      if (this.runWebGLPromise) {
        await this.runWebGLPromise;
      }
      this.hasRequestedAnimationFrame = false;
      if (this.xrIsPresenting) {
        return;
      }
      this.zerdeEventloopEvents.animationFrame();
      try {
        this.doWasmIo();
      } catch (e) {
        if (e instanceof Error && e.name === "RustPanic") {
          Atomics.store(wasmOnline, 0, 0);
          rpc.send(WorkerEvent.Panic, e);
        } else {
          throw e;
        }
      }
    });
  }

  // private runAsyncWebXRCheck(): void {
  //   this.xrCanPresent = false;
  //   this.xrIsPresenting = false;

  //   // ok this changes a bunch in how the renderflow works.
  //   // first thing we are going to do is get the vr displays.
  //   // @ts-ignore - Let's not worry about XR.
  //   const xrSystem = globalThis.navigator.xr;
  //   if (xrSystem) {
  //     xrSystem.isSessionSupported("immersive-vr").then((supported) => {
  //       if (supported) {
  //         this.xrCanPresent = true;
  //       }
  //     });
  //   } else {
  //     console.log("No webVR support found");
  //   }
  // }

  private xrStartPresenting(): void {
    // TODO(JP): Some day bring back XR support?
    // if (this.xr_can_present) {
    //     navigator.xr.requestSession('immersive-vr', {requiredFeatures: ['local-floor']}).then(xr_session => {
    //         let xr_layer = new XRWebGLLayer(xr_session, this.gl, {
    //             antialias: false,
    //             depth: true,
    //             stencil: false,
    //             alpha: false,
    //             ignoreDepthValues: false,
    //             framebufferScaleFactor: 1.5
    //         });
    //         xr_session.updateRenderState({baseLayer: xr_layer});
    //         xr_session.requestReferenceSpace("local-floor").then(xr_reference_space => {
    //             window.localStorage.setItem("xr_presenting", "true");
    //             this.xr_reference_space = xr_reference_space;
    //             this.xr_session = xr_session;
    //             this.xr_is_presenting = true;
    //             let first_on_resize = true;
    //             // read shit off the gamepad
    //             xr_session.gamepad;
    //             // lets start the loop
    //             let inputs = [];
    //             let alternate = false;
    //             let last_time;
    //             let xr_on_request_animation_frame = (time, xr_frame) => {
    //                 if (first_on_resize) {
    //                     this.on_screen_resize();
    //                     first_on_resize = false;
    //                 }
    //                 if (!this.xr_is_presenting) {
    //                     return;
    //                 }
    //                 this.xr_session.requestAnimationFrame(xr_on_request_animation_frame);
    //                 this.xr_pose = xr_frame.getViewerPose(this.xr_reference_space);
    //                 if (!this.xr_pose) {
    //                     return;
    //                 }
    //                 this.zerdeEventloopEvents.xr_update_inputs(xr_frame, xr_session, time, this.xr_pose, this.xr_reference_space)
    //                 this.zerdeEventloopEvents.animation_frame(time / 1000.0);
    //                 this.in_animation_frame = true;
    //                 let start = performance.now();
    //                 this.do_wasm_io();
    //                 this.in_animation_frame = false;
    //                 this.xr_pose = null;
    //                 //let new_time = performance.now();
    //                 //if (new_time - last_time > 13.) {
    //                 //    console.log(new_time - last_time);
    //                 // }
    //                 //last_time = new_time;
    //             }
    //             this.xr_session.requestAnimationFrame(xr_on_request_animation_frame);
    //             this.xr_session.addEventListener("end", () => {
    //                 window.localStorage.setItem("xr_presenting", "false");
    //                 this.xr_is_presenting = false;
    //                 this.on_screen_resize();
    //                 this.zerdeEventloopEvents.paint_dirty();
    //                 this.request_animation_frame();
    //             })
    //         })
    //     })
    // }
  }

  private xrStopPresenting(): void {
    // ignore for now
  }

  sendEventFromAnyThread(eventPtr: BigInt): void {
    // Prevent an infinite loop when calling this from an event handler.
    setTimeout(() => {
      try {
        this.zerdeEventloopEvents.sendEventFromAnyThread(eventPtr);
        this.doWasmIo();
      } catch (e) {
        if (e instanceof Error && e.name === "RustPanic") {
          Atomics.store(wasmOnline, 0, 0);
          rpc.send(WorkerEvent.Panic, e);
        } else {
          throw e;
        }
      }
    });
  }

  // Array of function id's wasm can call on us; `zelf` is pointer to WasmApp.
  // (It's not called `self` as to not overload https://developer.mozilla.org/en-US/docs/Web/API/Window/self)
  // Function names are suffixed with the index in the array, and annotated with
  // their name in cx_wasm32.rs, for easier matching.
  private sendFnTable: ((zelf: this) => void | boolean)[] = [
    // end
    function end0(_zelf) {
      return true;
    },
    // run_webgl
    function runWebGL1(zelf) {
      const zerdeParserPtr = zelf.zerdeParser.parseU64();
      if (zelf.webglRenderer) {
        zelf.webglRenderer.processMessages(Number(zerdeParserPtr));
        zelf.exports.deallocWasmMessage(zerdeParserPtr);
      } else {
        zelf.runWebGLPromise = rpc
          .send(WorkerEvent.RunWebGL, Number(zerdeParserPtr))
          .then(() => {
            zelf.exports.deallocWasmMessage(zerdeParserPtr);
            zelf.runWebGLPromise = undefined;
          });
      }
    },
    // log
    function log2(zelf) {
      console.log(zelf.zerdeParser.parseString());
    },
    // request_animation_frame
    function requestAnimationFrame3(zelf) {
      zelf.requestAnimationFrame();
    },
    // set_document_title
    function setDocumentTitle4(zelf) {
      zelf.setDocumentTitle(zelf.zerdeParser.parseString());
    },
    // set_mouse_cursor
    function setMouseCursor5(zelf) {
      zelf.setMouseCursor(zelf.zerdeParser.parseU32());
    },
    // show_text_ime
    function showTextIme6(zelf) {
      const x = zelf.zerdeParser.parseF32();
      const y = zelf.zerdeParser.parseF32();
      rpc.send(WorkerEvent.ShowTextIME, { x, y });
    },
    // hide_text_ime
    function hideTextIme7(_zelf) {
      // TODO(JP): doesn't seem to do anything, is that intentional?
    },
    // text_copy_response
    function textCopyResponse8(zelf) {
      const textCopyResponse = zelf.zerdeParser.parseString();
      rpc.send(WorkerEvent.TextCopyResponse, textCopyResponse);
    },
    // start_timer
    function startTimer9(zelf) {
      const repeats = zelf.zerdeParser.parseU32();
      const id = zelf.zerdeParser.parseF64();
      const interval = zelf.zerdeParser.parseF64();
      zelf.startTimer(id, interval, repeats);
    },
    // stop_timer
    function stopTimer10(zelf) {
      const id = zelf.zerdeParser.parseF64();
      zelf.stopTimer(id);
    },
    // xr_start_presenting
    function xrStartPresenting11(zelf) {
      zelf.xrStartPresenting();
    },
    // xr_stop_presenting
    function xrStopPresenting12(zelf) {
      zelf.xrStopPresenting();
    },
    // http_send
    function httpSend13(zelf) {
      const port = zelf.zerdeParser.parseU32();
      const signalId = zelf.zerdeParser.parseU32();
      const verb = zelf.zerdeParser.parseString();
      const path = zelf.zerdeParser.parseString();
      const proto = zelf.zerdeParser.parseString();
      const domain = zelf.zerdeParser.parseString();
      const contentType = zelf.zerdeParser.parseString();
      const body = zelf.zerdeParser.parseU8Slice();
      zelf.httpSend(
        verb,
        path,
        proto,
        domain,
        port,
        contentType,
        body,
        signalId
      );
    },
    // fullscreen
    function fullscreen14(_zelf) {
      rpc.send(WorkerEvent.Fullscreen);
    },
    // normalscreen
    function normalscreen15(_zelf) {
      rpc.send(WorkerEvent.Normalscreen);
    },
    // websocket_send
    function websocketSend16(zelf) {
      const url = zelf.zerdeParser.parseString();
      const data = zelf.zerdeParser.parseU8Slice();
      zelf.websocketSend(url, data);
    },
    // enable_global_file_drop_target
    function enableGlobalFileDropTarget17(zelf) {
      zelf.enableGlobalFileDropTarget();
    },
    // call_js
    function callJs18(zelf) {
      const fnName = zelf.zerdeParser.parseString();
      const params = zelf.zerdeParser.parseZapParams();
      if (fnName === "_zaplibReturnParams") {
        const callbackId = JSON.parse(params[0] as string);
        zelf.callRustAsyncPendingCallbacks[callbackId](params.slice(1));
        delete zelf.callRustAsyncPendingCallbacks[callbackId];
      } else {
        rpc.send(WorkerEvent.CallJs, { fnName, params });
      }
    },
  ];
}

rpc.receive(
  WorkerEvent.Init,
  ({
    wasmModule,
    offscreenCanvas,
    sizingData,
    baseUri,
    memory,
    taskWorkerSab,
    tlsAndStackData,
    appPtr,
    wasmOnline: _wasmOnline,
  }) => {
    wasmOnline = _wasmOnline;

    let wasmapp: WasmApp;
    return new Promise<void>((resolve, reject) => {
      const threadSpawn = (ctxPtr: BigInt) => {
        rpc.send(WorkerEvent.ThreadSpawn, {
          ctxPtr,
          tlsAndStackData: makeThreadLocalStorageAndStackDataOnExistingThread(
            wasmapp.exports
          ),
        });
      };

      const getExports = () => {
        return wasmapp.exports;
      };

      const fileHandles: FileHandle[] = [];

      const env = getWasmEnv({
        getExports,
        memory,
        taskWorkerSab,
        fileHandles,
        sendEventFromAnyThread: (eventPtr: BigInt) => {
          wasmapp.sendEventFromAnyThread(eventPtr);
        },
        threadSpawn,
        baseUri,
      });

      WebAssembly.instantiate(wasmModule, { env }).then((instance: any) => {
        const wasmExports = instance.exports as WasmExports;
        initThreadLocalStorageAndStackOtherWorkers(
          wasmExports,
          tlsAndStackData
        );
        wasmapp = new WasmApp({
          offscreenCanvas,
          wasmModule,
          wasmExports,
          memory,
          sizingData,
          baseUri,
          fileHandles,
          taskWorkerSab,
          appPtr,
        });
        wasmapp.init();
        resolve();
      }, reject);
    });
  }
);
