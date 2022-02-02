export type RpcMouseEvent = Pick<
  MouseEvent,
  "button" | "pageX" | "pageY" | "shiftKey" | "metaKey" | "ctrlKey" | "altKey"
>;
export const makeRpcMouseEvent = (event: MouseEvent): RpcMouseEvent => {
  return {
    pageX: event.pageX,
    pageY: event.pageY,
    button: event.button,
    shiftKey: event.shiftKey,
    ctrlKey: event.ctrlKey,
    metaKey: event.metaKey,
    altKey: event.altKey,
  };
};

type RpcTouch = Pick<Touch, "pageX" | "pageY" | "identifier">;
export type RpcTouchEvent = Pick<
  TouchEvent,
  "shiftKey" | "metaKey" | "ctrlKey" | "altKey"
> & { changedTouches: RpcTouch[] };
export const makeRpcTouchEvent = (event: TouchEvent): RpcTouchEvent => {
  return {
    shiftKey: event.shiftKey,
    ctrlKey: event.ctrlKey,
    metaKey: event.metaKey,
    altKey: event.altKey,
    changedTouches: Array.from(event.changedTouches).map((touch) => ({
      pageX: touch.pageX,
      pageY: touch.pageY,
      identifier: touch.identifier,
    })),
  };
};

export type RpcWheelEvent = Pick<
  WheelEvent & { wheelDeltaY?: number },
  | "timeStamp"
  | "deltaMode"
  | "deltaX"
  | "deltaY"
  | "wheelDeltaY"
  | "button"
  | "pageX"
  | "pageY"
  | "shiftKey"
  | "metaKey"
  | "ctrlKey"
  | "altKey"
>;
export const makeRpcWheelEvent = (event: WheelEvent): RpcWheelEvent => {
  return {
    pageX: event.pageX,
    pageY: event.pageY,
    button: event.button,
    timeStamp: event.timeStamp,
    deltaMode: event.deltaMode,
    deltaX: event.deltaX,
    deltaY: event.deltaY,
    // @ts-ignore - the wheelDeltaY API is non-standard
    wheelDeltaY: event.wheelDeltaY,
    shiftKey: event.shiftKey,
    ctrlKey: event.ctrlKey,
    metaKey: event.metaKey,
    altKey: event.altKey,
  };
};

export type RpcKeyboardEvent = Pick<
  KeyboardEvent,
  "keyCode" | "repeat" | "shiftKey" | "metaKey" | "ctrlKey" | "altKey"
>;
export const makeRpcKeyboardEvent = (
  event: KeyboardEvent
): RpcKeyboardEvent => {
  return {
    keyCode: event.keyCode,
    repeat: event.repeat,
    shiftKey: event.shiftKey,
    ctrlKey: event.ctrlKey,
    metaKey: event.metaKey,
    altKey: event.altKey,
  };
};
