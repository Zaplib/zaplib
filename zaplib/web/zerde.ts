// Zerde is our lightweight manual serialization/deserialization system.
//
// Keep in sync with zerde.rs, and see there for more information.

import { RustZapParam, ZapParamType } from "types";

type GrowCallback = (
  _buffer: ArrayBuffer,
  byteOffset: number,
  newBytes: number
) => {
  buffer: ArrayBuffer;
  byteOffset: number;
};

// Construct a buffer that can be read in Rust, using the corresponding `ZerderParser` struct in Rust.
export class ZerdeBuilder {
  private _buffer: ArrayBuffer;
  private _byteOffset: number;
  private _slots: number;
  private _growCallback: GrowCallback;
  private _used: number;
  private _f32!: Float32Array;
  private _u32!: Uint32Array;
  private _f64!: Float64Array;
  private _u64!: BigUint64Array;

  constructor({
    buffer,
    byteOffset,
    slots,
    growCallback,
  }: {
    buffer: ArrayBuffer;
    byteOffset: number;
    slots: number;
    growCallback: GrowCallback;
  }) {
    this._buffer = buffer;
    this._byteOffset = byteOffset;
    this._slots = slots;
    this._growCallback = growCallback;
    this._used = 2; // Skip 8 byte header which contains the size.
    this._updateRefs();
  }

  _updateRefs(): void {
    this._f32 = new Float32Array(this._buffer, this._byteOffset, this._slots);
    this._u32 = new Uint32Array(this._buffer, this._byteOffset, this._slots);
    this._f64 = new Float64Array(
      this._buffer,
      this._byteOffset,
      this._slots >> 1
    );
    this._u64 = new BigUint64Array(
      this._buffer,
      this._byteOffset,
      this._slots >> 1
    );
    this._u64[0] = BigInt(this._slots) * BigInt(4); // Write size to header.
  }

  _fit(slots: number): number {
    if (this._used + slots > this._slots) {
      let newSlots = Math.max(this._used + slots, this._slots * 2); // Exponential growth
      if (newSlots & 1) newSlots++; // 64-bit align it
      const newBytes = newSlots * 4;
      const { buffer, byteOffset } = this._growCallback(
        this._buffer,
        this._byteOffset,
        newBytes
      );
      this._buffer = buffer;
      this._byteOffset = byteOffset;
      this._slots = newSlots;
      this._updateRefs();
    }
    const pos = this._used;
    this._used += slots;
    return pos;
  }

  sendF32(value: number): void {
    const pos = this._fit(1);
    this._f32[pos] = value;
  }

  sendU32(value: number): void {
    const pos = this._fit(1);
    this._u32[pos] = value;
  }

  sendF64(value: number): void {
    if (this._used & 1) {
      // 64-bit alignment.
      const pos = this._fit(3) + 1;
      this._f64[pos >> 1] = value;
    } else {
      const pos = this._fit(2);
      this._f64[pos >> 1] = value;
    }
  }

  sendU64(value: BigInt): void {
    if (this._used & 1) {
      // 64-bit alignment.
      const pos = this._fit(3) + 1;
      this._u64[pos >> 1] = value as bigint;
    } else {
      const pos = this._fit(2);
      this._u64[pos >> 1] = value as bigint;
    }
  }

  sendString(str: string): void {
    let pos = this._fit(str.length + 1);
    this._u32[pos++] = str.length;
    for (let i = 0; i < str.length; i++) {
      this._u32[pos++] = str.charCodeAt(i);
    }
  }

  getData(): { buffer: ArrayBuffer; byteOffset: number } {
    return { buffer: this._buffer, byteOffset: this._byteOffset };
  }
}

export class ZerdeParser {
  private _memory: WebAssembly.Memory;
  private _usedSlots: number;
  private _f32: Float32Array;
  private _u32: Uint32Array;
  private _f64: Float64Array;
  private _u64: BigUint64Array;

  constructor(memory: WebAssembly.Memory, zerdePtr: number) {
    this._memory = memory;
    // set up local shortcuts to the zerde memory chunk for faster parsing
    this._usedSlots = 2; // skip the 8 byte header
    this._f32 = new Float32Array(this._memory.buffer, zerdePtr);
    this._u32 = new Uint32Array(this._memory.buffer, zerdePtr);
    this._f64 = new Float64Array(this._memory.buffer, zerdePtr);
    this._u64 = new BigUint64Array(this._memory.buffer, zerdePtr);
  }

  parseU32(): number {
    return this._u32[this._usedSlots++];
  }

  parseF32(): number {
    return this._f32[this._usedSlots++];
  }

  parseF64(): number {
    if (this._usedSlots & 1) {
      // 64-bit alignment.
      this._usedSlots++;
    }
    const ret = this._f64[this._usedSlots >> 1];
    this._usedSlots += 2;
    return ret;
  }

  parseU64(): BigInt {
    if (this._usedSlots & 1) {
      // 64-bit alignment.
      this._usedSlots++;
    }
    const ret = this._u64[this._usedSlots >> 1];
    this._usedSlots += 2;
    return ret;
  }

  parseString(): string {
    let str = "";
    const len = this.parseU32();
    for (let i = 0; i < len; i++) {
      const c = this.parseU32();
      if (c != 0) str += String.fromCharCode(c);
    }
    return str;
  }

  parseU8Slice(): Uint8Array {
    const u8Len = this.parseU32();
    const len = u8Len >> 2;
    const data = new Uint8Array(u8Len);
    const spare = u8Len & 3;
    for (let i = 0; i < len; i++) {
      const u8Pos = i << 2;
      const u32 = this.parseU32();
      data[u8Pos + 0] = u32 & 0xff;
      data[u8Pos + 1] = (u32 >> 8) & 0xff;
      data[u8Pos + 2] = (u32 >> 16) & 0xff;
      data[u8Pos + 3] = (u32 >> 24) & 0xff;
    }
    const u8Pos = len << 2;
    if (spare == 1) {
      const u32 = this.parseU32();
      data[u8Pos + 0] = u32 & 0xff;
    } else if (spare == 2) {
      const u32 = this.parseU32();
      data[u8Pos + 0] = u32 & 0xff;
      data[u8Pos + 1] = (u32 >> 8) & 0xff;
    } else if (spare == 3) {
      const u32 = this.parseU32();
      data[u8Pos + 0] = u32 & 0xff;
      data[u8Pos + 1] = (u32 >> 8) & 0xff;
      data[u8Pos + 2] = (u32 >> 16) & 0xff;
    }
    return data;
  }

  parseZapParams(): RustZapParam[] {
    const len = this.parseU32();
    const params: RustZapParam[] = [];
    for (let i = 0; i < len; ++i) {
      const paramType: ZapParamType = this.parseU32();
      if (paramType === ZapParamType.String) {
        params.push(this.parseString());
      } else if (
        paramType === ZapParamType.ReadOnlyU8Buffer ||
        paramType === ZapParamType.ReadOnlyU32Buffer ||
        paramType === ZapParamType.ReadOnlyF32Buffer
      ) {
        const bufferPtr = this.parseU32();
        const bufferLen = this.parseU32();
        const arcPtr = this.parseU32();

        params.push({
          paramType,
          bufferPtr,
          bufferLen,
          arcPtr,
          readonly: true,
        });
      } else if (
        paramType === ZapParamType.U8Buffer ||
        paramType === ZapParamType.U32Buffer ||
        paramType === ZapParamType.F32Buffer
      ) {
        const bufferPtr = this.parseU32();
        const bufferLen = this.parseU32();
        const bufferCap = this.parseU32();
        params.push({
          paramType,
          bufferPtr,
          bufferLen,
          bufferCap,
          readonly: false,
        });
      } else {
        throw new Error(`Unknown ZapParam type: ${paramType}`);
      }
    }
    return params;
  }
}
