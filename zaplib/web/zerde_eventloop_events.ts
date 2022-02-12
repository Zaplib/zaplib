import { createWasmBuffer, getZapParamType, makeZerdeBuilder } from "./common";
import { Pointer, PointerScroll, WasmApp } from "./main_worker";
import {
  TextareaEventKeyDown,
  TextareaEventKeyUp,
  TextareaEventTextInput,
} from "./make_textarea";
import {
  FileHandle,
  PostMessageTypedArray,
  ZapArray,
  ZapParamType,
} from "./types";
import { gitSha } from "./type_of_runtime";
import { ZerdeBuilder } from "./zerde";
import { zerdeKeyboardHandlers } from "./zerde_keyboard_handlers";

// These constants must be kept in sync with the ones in main/src/cx_wasm32.rs
const MSG_TYPE_END = 0;
const MSG_TYPE_INIT = 1;
const MSG_TYPE_RESIZE = 4;
const MSG_TYPE_ANIMATION_FRAME = 5;
const MSG_TYPE_POINTER_DOWN = 6;
const MSG_TYPE_POINTER_UP = 7;
const MSG_TYPE_POINTER_MOVE = 8;
const MSG_TYPE_POINTER_HOVER = 9;
const MSG_TYPE_POINTER_SCROLL = 10;
const MSG_TYPE_POINTER_OUT = 11;
const MSG_TYPE_TIMER_FIRED = 18;
const MSG_TYPE_WINDOW_FOCUS = 19;
const MSG_TYPE_PAINT_DIRTY = 21;
const MSG_TYPE_HTTP_SEND_RESPONSE = 22;
const MSG_TYPE_WEBSOCKET_MESSAGE = 23;
const MSG_TYPE_WEBSOCKET_ERROR = 24;
const MSG_TYPE_APP_OPEN_FILES = 25;
const MSG_TYPE_SEND_EVENT_FROM_ANY_THREAD = 26;
const MSG_TYPE_DRAG_ENTER = 27;
const MSG_TYPE_DRAG_LEAVE = 28;
const MSG_TYPE_DRAG_OVER = 29;
const MSG_TYPE_CALL_RUST = 30;

// A set of events. Each event starts with a u32 representing the event type, with 0 indicating the end. And
// it is prefixed by a timestamp.
export class ZerdeEventloopEvents {
  private _wasmApp: WasmApp;
  private _zerdeBuilder: ZerdeBuilder;

  constructor(wasmApp: WasmApp) {
    this._wasmApp = wasmApp;
    this._zerdeBuilder = makeZerdeBuilder(wasmApp.memory, wasmApp.exports);
    this._zerdeBuilder.sendF64(0); // Fit an f64 for the timestamp of when we send the message.
  }

  getWasmApp(): WasmApp {
    return this._wasmApp;
  }

  createWasmBuffer(data: ZapArray): number {
    return createWasmBuffer(this._wasmApp.memory, this._wasmApp.exports, data);
  }

  createArcVec(vecPtr: number, data: ZapArray): number {
    return Number(
      this._wasmApp.exports.createArcVec(
        BigInt(vecPtr),
        BigInt(data.length),
        BigInt(getZapParamType(data, true))
      )
    );
  }

  init(info: {
    width: number;
    height: number;
    dpiFactor: number;
    xrCanPresent: boolean;
    canFullscreen: boolean;
    xrIsPresenting: false;
  }): void {
    this._zerdeBuilder.sendU32(MSG_TYPE_INIT);
    this._zerdeBuilder.sendF32(info.width);
    this._zerdeBuilder.sendF32(info.height);
    this._zerdeBuilder.sendF32(info.dpiFactor);
    this._zerdeBuilder.sendU32(info.xrCanPresent ? 1 : 0);
    this._zerdeBuilder.sendU32(info.canFullscreen ? 1 : 0);
    if (process.env.NODE_ENV === "production") {
      this._zerdeBuilder.sendString(gitSha);
    } else {
      this._zerdeBuilder.sendString("development");
    }
  }

  resize(info: {
    width: number;
    height: number;
    dpiFactor: number;
    xrCanPresent: boolean;
    isFullscreen: boolean;
    xrIsPresenting: boolean;
  }): void {
    this._zerdeBuilder.sendU32(MSG_TYPE_RESIZE);
    this._zerdeBuilder.sendF32(info.width);
    this._zerdeBuilder.sendF32(info.height);
    this._zerdeBuilder.sendF32(info.dpiFactor);
    this._zerdeBuilder.sendU32(info.xrIsPresenting ? 1 : 0);
    this._zerdeBuilder.sendU32(info.xrCanPresent ? 1 : 0);
    this._zerdeBuilder.sendU32(info.isFullscreen ? 1 : 0);
  }

  animationFrame(): void {
    this._zerdeBuilder.sendU32(MSG_TYPE_ANIMATION_FRAME);
  }

  pointerDown(pointer: Pointer): void {
    this._zerdeBuilder.sendU32(MSG_TYPE_POINTER_DOWN);
    this._zerdeBuilder.sendF32(pointer.x);
    this._zerdeBuilder.sendF32(pointer.y);
    this._zerdeBuilder.sendU32(pointer.button);
    this._zerdeBuilder.sendU32(pointer.digit);
    this._zerdeBuilder.sendU32(pointer.touch ? 1 : 0);
    this._zerdeBuilder.sendU32(pointer.modifiers);
    this._zerdeBuilder.sendF64(pointer.time);
  }

  pointerUp(pointer: Pointer): void {
    this._zerdeBuilder.sendU32(MSG_TYPE_POINTER_UP);
    this._zerdeBuilder.sendF32(pointer.x);
    this._zerdeBuilder.sendF32(pointer.y);
    this._zerdeBuilder.sendU32(pointer.button);
    this._zerdeBuilder.sendU32(pointer.digit);
    this._zerdeBuilder.sendU32(pointer.touch ? 1 : 0);
    this._zerdeBuilder.sendU32(pointer.modifiers);
    this._zerdeBuilder.sendF64(pointer.time);
  }

  pointerMove(pointer: Pointer): void {
    this._zerdeBuilder.sendU32(MSG_TYPE_POINTER_MOVE);
    this._zerdeBuilder.sendF32(pointer.x);
    this._zerdeBuilder.sendF32(pointer.y);
    this._zerdeBuilder.sendU32(pointer.digit);
    this._zerdeBuilder.sendU32(pointer.touch ? 1 : 0);
    this._zerdeBuilder.sendU32(pointer.modifiers);
    this._zerdeBuilder.sendF64(pointer.time);
  }

  pointerHover(pointer: Pointer): void {
    this._zerdeBuilder.sendU32(MSG_TYPE_POINTER_HOVER);
    this._zerdeBuilder.sendF32(pointer.x);
    this._zerdeBuilder.sendF32(pointer.y);
    this._zerdeBuilder.sendU32(pointer.modifiers);
    this._zerdeBuilder.sendF64(pointer.time);
  }

  pointerScroll(pointer: PointerScroll): void {
    this._zerdeBuilder.sendU32(MSG_TYPE_POINTER_SCROLL);
    this._zerdeBuilder.sendF32(pointer.x);
    this._zerdeBuilder.sendF32(pointer.y);
    this._zerdeBuilder.sendF32(pointer.scrollX);
    this._zerdeBuilder.sendF32(pointer.scrollY);
    this._zerdeBuilder.sendU32(pointer.isWheel ? 1 : 0);
    this._zerdeBuilder.sendU32(pointer.modifiers);
    this._zerdeBuilder.sendF64(pointer.time);
  }

  pointerOut(pointer: Pointer): void {
    this._zerdeBuilder.sendU32(MSG_TYPE_POINTER_OUT);
    this._zerdeBuilder.sendF32(pointer.x);
    this._zerdeBuilder.sendF32(pointer.y);
    this._zerdeBuilder.sendU32(pointer.modifiers);
    this._zerdeBuilder.sendF64(pointer.time);
  }

  keyDown(data: TextareaEventKeyDown): void {
    zerdeKeyboardHandlers.keyDown(this._zerdeBuilder, data);
  }

  keyUp(data: TextareaEventKeyUp): void {
    zerdeKeyboardHandlers.keyUp(this._zerdeBuilder, data);
  }

  textInput(data: TextareaEventTextInput): void {
    zerdeKeyboardHandlers.textInput(this._zerdeBuilder, data);
  }

  textCopy(): void {
    zerdeKeyboardHandlers.textCopy(this._zerdeBuilder);
  }

  timerFired(id: number): void {
    this._zerdeBuilder.sendU32(MSG_TYPE_TIMER_FIRED);
    this._zerdeBuilder.sendF64(id);
  }

  windowFocus(isFocus: boolean): void {
    // TODO CALL THIS
    this._zerdeBuilder.sendU32(MSG_TYPE_WINDOW_FOCUS);
    this._zerdeBuilder.sendU32(isFocus ? 1 : 0);
  }

  xrUpdateHead(_inputsLen: unknown, _time: unknown): void {
    //this._zerde_builder.send_f64(time);
  }

  xrUpdateInputs(
    _xrFrame: unknown,
    _xrSessions: unknown,
    _time: unknown,
    _xrPose: unknown,
    _xrReferenceSpace: unknown
  ): void {
    // send_pose_transform(pose_transform) {
    //     let pos = this._fit(7)
    //     let inv_orient = pose_transform.inverse.orientation;
    //     this._f32[pos++] = inv_orient.x;
    //     this._f32[pos++] = inv_orient.y;
    //     this._f32[pos++] = inv_orient.z;
    //     this._f32[pos++] = inv_orient.w;
    //     let tpos = pose_transform.position;
    //     this._f32[pos++] = tpos.x;
    //     this._f32[pos++] = tpos.y;
    //     this._f32[pos++] = tpos.z;
    // }
    // let input_len = xr_session.inputSources.length;
    // let pos = this.fit(2);
    // this.mu32[pos ++] = 20;
    // this.mu32[pos ++] = input_len;
    // this._zerde_builder.send_f64(time / 1000.0);
    // this.send_pose_transform(xr_pose.transform);
    // for (let i = 0; i < input_len; i ++) {
    //     let input = xr_session.inputSources[i];
    //     let grip_pose = xr_frame.getPose(input.gripSpace, xr_reference_space);
    //     let ray_pose = xr_frame.getPose(input.targetRaySpace, xr_reference_space);
    //     if (grip_pose == null || ray_pose == null) {
    //         let pos = this.fit(1);
    //         this.mu32[pos ++] = 0;
    //         continue
    //     }
    //     let pos = this.fit(1);
    //     this.mu32[pos ++] = 1;
    //     this.send_pose_transform(grip_pose.transform)
    //     this.send_pose_transform(ray_pose.transform)
    //     let buttons = input.gamepad.buttons;
    //     let axes = input.gamepad.axes;
    //     let buttons_len = buttons.length;
    //     let axes_len = axes.length;
    //     pos = this.fit(3 + buttons_len * 2 + axes_len);
    //     this.mu32[pos ++] = input.handedness == "left"? 1: input.handedness == "right"? 2: 0;
    //     this.mu32[pos ++] = buttons_len;
    //     for (let i = 0; i < buttons_len; i ++) {
    //         this.mu32[pos ++] = buttons[i].pressed? 1: 0;
    //         this.mf32[pos ++] = buttons[i].value;
    //     }
    //     this.mu32[pos ++] = axes_len;
    //     for (let i = 0; i < axes_len; i ++) {
    //         this.mf32[pos ++] = axes[i]
    //     }
    // }
  }

  paintDirty(_time: unknown, _frameData: unknown): void {
    this._zerdeBuilder.sendU32(MSG_TYPE_PAINT_DIRTY);
  }

  httpSendResponse(signalId: number, success: 1 | 2): void {
    this._zerdeBuilder.sendU32(MSG_TYPE_HTTP_SEND_RESPONSE);
    this._zerdeBuilder.sendU32(signalId);
    this._zerdeBuilder.sendU32(success);
  }

  sendEventFromAnyThread(eventPtr: BigInt): void {
    this._zerdeBuilder.sendU32(MSG_TYPE_SEND_EVENT_FROM_ANY_THREAD);
    this._zerdeBuilder.sendU64(eventPtr);
  }

  websocketMessage(url: string, data: ArrayBuffer): void {
    const vecLen = data.byteLength;
    const vecPtr = this.createWasmBuffer(new Uint8Array(data));
    this._zerdeBuilder.sendU32(MSG_TYPE_WEBSOCKET_MESSAGE);
    this._zerdeBuilder.sendU32(vecPtr);
    this._zerdeBuilder.sendU32(vecLen);
    this._zerdeBuilder.sendString(url);
  }

  websocketError(url: string, error: string): void {
    this._zerdeBuilder.sendU32(MSG_TYPE_WEBSOCKET_ERROR);
    this._zerdeBuilder.sendString(url);
    this._zerdeBuilder.sendString(error);
  }

  appOpenFiles(fileHandles: FileHandle[]): void {
    this._zerdeBuilder.sendU32(MSG_TYPE_APP_OPEN_FILES);
    this._zerdeBuilder.sendU32(fileHandles.length);
    for (const fileHandle of fileHandles) {
      this._zerdeBuilder.sendU32(fileHandle.id);
      this._zerdeBuilder.sendU64(BigInt(fileHandle.file.size));
      this._zerdeBuilder.sendString(fileHandle.basename);
    }
  }

  dragenter(): void {
    this._zerdeBuilder.sendU32(MSG_TYPE_DRAG_ENTER);
  }

  dragleave(): void {
    this._zerdeBuilder.sendU32(MSG_TYPE_DRAG_LEAVE);
  }

  dragover(x: number, y: number): void {
    this._zerdeBuilder.sendU32(MSG_TYPE_DRAG_OVER);
    this._zerdeBuilder.sendU32(x);
    this._zerdeBuilder.sendU32(y);
  }

  callRust(
    name: string,
    params: (string | ZapArray | PostMessageTypedArray)[],
    callbackId: number
  ): void {
    this._zerdeBuilder.sendU32(MSG_TYPE_CALL_RUST);
    this._zerdeBuilder.sendString(name);
    this._zerdeBuilder.sendU32(params.length);
    for (const param of params) {
      if (typeof param === "string") {
        this._zerdeBuilder.sendU32(ZapParamType.String);
        this._zerdeBuilder.sendString(param);
      } else {
        if ("bufferData" in param) {
          this._zerdeBuilder.sendU32(param.bufferData.paramType);
          if (param.bufferData.readonly) {
            this._zerdeBuilder.sendU32(param.bufferData.arcPtr);
          } else {
            this._zerdeBuilder.sendU32(param.bufferData.bufferPtr);
            this._zerdeBuilder.sendU32(param.bufferData.bufferLen);
            this._zerdeBuilder.sendU32(param.bufferData.bufferCap);
          }
        } else {
          const vecLen = param.byteLength;
          const vecPtr = this.createWasmBuffer(param);
          this._zerdeBuilder.sendU32(getZapParamType(param, false));
          this._zerdeBuilder.sendU32(vecPtr);
          this._zerdeBuilder.sendU32(vecLen);
          this._zerdeBuilder.sendU32(vecLen);
        }
      }
    }
    this._zerdeBuilder.sendU32(callbackId);
  }

  end(): number {
    this._zerdeBuilder.sendU32(MSG_TYPE_END);

    const { buffer, byteOffset } = this._zerdeBuilder.getData();

    // Fill in the current timestamp that we reserved space for at the beginning.
    new Float64Array(buffer, byteOffset, 2)[1] = performance.now() / 1000.0;

    return byteOffset;
  }
}
