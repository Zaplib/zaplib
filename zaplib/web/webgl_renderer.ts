import { assertNotNull } from "common";
import {
  ShaderAttributes,
  SizingData,
  Texture,
  Uniform,
  UniformType,
} from "types";
import { ZerdeParser } from "zerde";

type UniformLocation = {
  name: string;
  offset: number;
  ty: string;
  loc: WebGLUniformLocation | null;
  fn: WebGLRenderer["uniformFnTable"][number];
};

export class WebGLRenderer {
  private canvas: HTMLCanvasElement | OffscreenCanvas;
  private memory: WebAssembly.Memory;
  private sizingData: SizingData;
  private shaders: {
    geomAttribs: ReturnType<WebGLRenderer["getAttribLocations"]>;
    instAttribs: ReturnType<WebGLRenderer["getAttribLocations"]>;
    passUniforms: ReturnType<WebGLRenderer["getUniformLocations"]>;
    viewUniforms: ReturnType<WebGLRenderer["getUniformLocations"]>;
    drawUniforms: ReturnType<WebGLRenderer["getUniformLocations"]>;
    userUniforms: ReturnType<WebGLRenderer["getUniformLocations"]>;
    textureSlots: ReturnType<WebGLRenderer["getUniformLocations"]>;
    instanceSlots: number;
    program: WebGLProgram;
    ash: ShaderAttributes;
  }[];
  private indexBuffers: { glBuf: WebGLBuffer; length: number }[];
  private arrayBuffers: { glBuf: WebGLBuffer; length: number }[];
  private vaos: {
    glVao: WebGLVertexArrayObjectOES;
    geomIbId: number;
    geomVbId: number;
    instVbId: number;
  }[];
  private textures: Texture[];
  private framebuffers: WebGLFramebuffer[];
  private gl: WebGLRenderingContext;
  // eslint-disable-next-line camelcase
  private OESVertexArrayObject!: OES_vertex_array_object;
  // eslint-disable-next-line camelcase
  private ANGLEInstancedArrays!: ANGLE_instanced_arrays;
  private targetWidth: number;
  private targetHeight: number;
  private clearFlags: number;
  private clearR: number;
  private clearG: number;
  private clearB: number;
  private clearA: number;
  private clearDepth: number;

  private zerdeParser!: ZerdeParser;
  private basef32!: Float32Array;
  private baseu32!: Uint32Array;

  constructor(
    canvas: HTMLCanvasElement | OffscreenCanvas,
    memory: WebAssembly.Memory,
    sizingData: SizingData,
    incompatibleBrowserCallback: () => void
  ) {
    this.canvas = canvas;
    this.memory = memory;
    this.sizingData = sizingData;

    this.shaders = [];
    this.indexBuffers = [];
    this.arrayBuffers = [];
    this.vaos = [];
    this.textures = [];
    this.framebuffers = [];

    this.targetWidth = 0;
    this.targetHeight = 0;
    this.clearFlags = 0;
    this.clearR = 0;
    this.clearG = 0;
    this.clearB = 0;
    this.clearA = 0;
    this.clearDepth = 0;
    // this.isMainCanvas = false;

    const options = {
      preferLowPowerToHighPerformance: true,
      // xrCompatible: true // TODO(JP): Bring back some day?
    };
    // @ts-ignore - TODO(Paras): Get proper support for OffscreenCanvas
    this.gl =
      // @ts-ignore
      canvas.getContext("webgl", options) ||
      // @ts-ignore
      canvas.getContext("webgl-experimental", options) ||
      // @ts-ignore
      canvas.getContext("experimental-webgl", options);

    if (!this.gl) {
      incompatibleBrowserCallback();
      return;
    }

    this.OESVertexArrayObject = assertNotNull(
      this.gl.getExtension("OES_vertex_array_object")
    );
    this.ANGLEInstancedArrays = assertNotNull(
      this.gl.getExtension("ANGLE_instanced_arrays")
    );
    this.gl.getExtension("OES_standard_derivatives");
    this.gl.getExtension("OES_element_index_uint");
    this.resize(sizingData);
  }

  processMessages(zerdeParserPtr: number): void {
    this.zerdeParser = new ZerdeParser(this.memory, zerdeParserPtr);

    this.basef32 = new Float32Array(this.memory.buffer);
    this.baseu32 = new Uint32Array(this.memory.buffer);

    // eslint-disable-next-line no-constant-condition
    while (true) {
      const msgType = this.zerdeParser.parseU32();
      if (this.sendFnTable[msgType](this)) {
        break;
      }
    }
  }

  resize(sizingData: SizingData): void {
    this.sizingData = sizingData;
    this.canvas.width = sizingData.width * sizingData.dpiFactor;
    this.canvas.height = sizingData.height * sizingData.dpiFactor;
  }

  private getAttribLocations(
    program: WebGLProgram,
    base: string,
    slots: number
  ): {
    loc: number;
    offset: number;
    size: number;
    stride: number;
  }[] {
    const gl = this.gl;
    const attribLocs = [];
    let attribs = slots >> 2;
    if ((slots & 3) != 0) attribs++;
    for (let i = 0; i < attribs; i++) {
      let size = slots - i * 4;
      if (size > 4) size = 4;
      attribLocs.push({
        loc: gl.getAttribLocation(program, base + i),
        offset: i * 16,
        size: size,
        stride: slots * 4,
      });
    }
    return attribLocs;
  }

  private getUniformLocations(
    program: WebGLProgram,
    uniforms: Uniform[]
  ): UniformLocation[] {
    const gl = this.gl;
    const uniformLocs: UniformLocation[] = [];
    let offset = 0;
    for (let i = 0; i < uniforms.length; i++) {
      const uniform = uniforms[i];
      // lets align the uniform
      const slots = uniformSizeTable[uniform.ty];

      if ((offset & 3) != 0 && (offset & 3) + slots > 4) {
        // goes over the boundary
        offset += 4 - (offset & 3); // make jump to new slot
      }
      uniformLocs.push({
        name: uniform.name,
        offset: offset << 2,
        ty: uniform.ty,
        loc: gl.getUniformLocation(program, uniform.name),
        fn: this.uniformFnTable[uniform.ty],
      });
      offset += slots;
    }
    return uniformLocs;
  }

  private compileWebGLShader(ash: ShaderAttributes): void {
    const gl = this.gl;
    const vsh = assertNotNull(gl.createShader(gl.VERTEX_SHADER));

    gl.shaderSource(vsh, ash.vertex);
    gl.compileShader(vsh);
    if (!gl.getShaderParameter(vsh, gl.COMPILE_STATUS)) {
      console.log(gl.getShaderInfoLog(vsh), addLineNumbersToString(ash.vertex));
    }

    // compile pixelshader
    const fsh = assertNotNull(gl.createShader(gl.FRAGMENT_SHADER));
    gl.shaderSource(fsh, ash.fragment);
    gl.compileShader(fsh);
    if (!gl.getShaderParameter(fsh, gl.COMPILE_STATUS)) {
      console.log(
        gl.getShaderInfoLog(fsh),
        addLineNumbersToString(ash.fragment)
      );
    }

    const program = assertNotNull(gl.createProgram());
    gl.attachShader(program, vsh);
    gl.attachShader(program, fsh);
    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      console.log(
        gl.getProgramInfoLog(program),
        addLineNumbersToString(ash.vertex),
        addLineNumbersToString(ash.fragment)
      );
    }
    // fetch all attribs and uniforms
    this.shaders[ash.shaderId] = {
      geomAttribs: this.getAttribLocations(
        program,
        "mpsc_packed_geometry_",
        ash.geometrySlots
      ),
      instAttribs: this.getAttribLocations(
        program,
        "mpsc_packed_instance_",
        ash.instanceSlots
      ),
      passUniforms: this.getUniformLocations(program, ash.passUniforms),
      viewUniforms: this.getUniformLocations(program, ash.viewUniforms),
      drawUniforms: this.getUniformLocations(program, ash.drawUniforms),
      userUniforms: this.getUniformLocations(program, ash.userUniforms),
      textureSlots: this.getUniformLocations(program, ash.textureSlots),
      instanceSlots: ash.instanceSlots,
      program: program,
      ash: ash,
    };
  }

  private allocArrayBuffer(arrayBufferId: number, array: Float32Array): void {
    const gl = this.gl;
    let buf = this.arrayBuffers[arrayBufferId];
    if (buf === undefined) {
      buf = this.arrayBuffers[arrayBufferId] = {
        glBuf: assertNotNull(gl.createBuffer()),
        length: array.length,
      };
    } else {
      buf.length = array.length;
    }
    gl.bindBuffer(gl.ARRAY_BUFFER, buf.glBuf);
    gl.bufferData(gl.ARRAY_BUFFER, array, gl.STATIC_DRAW);
    gl.bindBuffer(gl.ARRAY_BUFFER, null);
  }

  private allocIndexBuffer(indexBufferId: number, array: Uint32Array): void {
    const gl = this.gl;

    let buf = this.indexBuffers[indexBufferId];
    if (buf === undefined) {
      buf = this.indexBuffers[indexBufferId] = {
        glBuf: assertNotNull(gl.createBuffer()),
        length: array.length,
      };
    } else {
      buf.length = array.length;
    }
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, buf.glBuf);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, array, gl.STATIC_DRAW);
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, null);
  }

  private allocVao(
    vaoId: number,
    shaderId: number,
    geomIbId: number,
    geomVbId: number,
    instVbId: number
  ): void {
    const gl = this.gl;
    const oldVao = this.vaos[vaoId];
    if (oldVao) {
      this.OESVertexArrayObject.deleteVertexArrayOES(oldVao.glVao);
    }
    const glVao = assertNotNull(
      this.OESVertexArrayObject.createVertexArrayOES()
    );
    const vao = (this.vaos[vaoId] = { glVao, geomIbId, geomVbId, instVbId });

    this.OESVertexArrayObject.bindVertexArrayOES(vao.glVao);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.arrayBuffers[geomVbId].glBuf);

    const shader = this.shaders[shaderId];

    for (let i = 0; i < shader.geomAttribs.length; i++) {
      const attr = shader.geomAttribs[i];
      if (attr.loc < 0) {
        continue;
      }
      gl.vertexAttribPointer(
        attr.loc,
        attr.size,
        gl.FLOAT,
        false,
        attr.stride,
        attr.offset
      );
      gl.enableVertexAttribArray(attr.loc);
      this.ANGLEInstancedArrays.vertexAttribDivisorANGLE(attr.loc, 0);
    }

    gl.bindBuffer(gl.ARRAY_BUFFER, this.arrayBuffers[instVbId].glBuf);
    for (let i = 0; i < shader.instAttribs.length; i++) {
      const attr = shader.instAttribs[i];
      if (attr.loc < 0) {
        continue;
      }
      gl.vertexAttribPointer(
        attr.loc,
        attr.size,
        gl.FLOAT,
        false,
        attr.stride,
        attr.offset
      );
      gl.enableVertexAttribArray(attr.loc);
      this.ANGLEInstancedArrays.vertexAttribDivisorANGLE(attr.loc, 1);
    }

    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.indexBuffers[geomIbId].glBuf);
    this.OESVertexArrayObject.bindVertexArrayOES(null);
  }

  private drawCall(
    shaderId: number,
    vaoId: number,
    passUniformsPtr: number,
    viewUniformsPtr: number,
    drawUniformsPtr: number,
    userUniformsPtr: number,
    texturesPtr: number
  ): void {
    const gl = this.gl;

    const shader = this.shaders[shaderId];
    gl.useProgram(shader.program);

    const vao = this.vaos[vaoId];

    this.OESVertexArrayObject.bindVertexArrayOES(vao.glVao);

    const indexBuffer = this.indexBuffers[vao.geomIbId];
    const instanceBuffer = this.arrayBuffers[vao.instVbId];
    // set up uniforms TODO do this a bit more incremental based on uniform layer
    // also possibly use webGL2 uniform buffers. For now this will suffice for webGL 1 compat
    const passUniforms = shader.passUniforms;
    // if vr_presenting

    const viewUniforms = shader.viewUniforms;
    for (let i = 0; i < viewUniforms.length; i++) {
      const uni = viewUniforms[i];
      uni.fn(this, uni.loc, uni.offset + viewUniformsPtr);
    }
    const drawUniforms = shader.drawUniforms;
    for (let i = 0; i < drawUniforms.length; i++) {
      const uni = drawUniforms[i];
      uni.fn(this, uni.loc, uni.offset + drawUniformsPtr);
    }
    const userUniforms = shader.userUniforms;
    for (let i = 0; i < userUniforms.length; i++) {
      const uni = userUniforms[i];
      uni.fn(this, uni.loc, uni.offset + userUniformsPtr);
    }
    const textureSlots = shader.textureSlots;
    for (let i = 0; i < textureSlots.length; i++) {
      const texSlot = textureSlots[i];
      const texId = this.baseu32[(texturesPtr >> 2) + i];
      const texObj = this.textures[texId];
      gl.activeTexture(gl.TEXTURE0 + i);
      gl.bindTexture(gl.TEXTURE_2D, texObj);
      gl.uniform1i(texSlot.loc, i);
    }
    const indices = indexBuffer.length;
    const instances = instanceBuffer.length / shader.instanceSlots;

    // if (this.isMainCanvas && xrIsPresenting) {
    // for (let i = 3; i < pass_uniforms.length; i ++) {
    //     let uni = pass_uniforms[i];
    //     uni.fn(this, uni.loc, uni.offset + pass_uniforms_ptr);
    // }
    // // the first 2 matrices are project and view
    // let left_viewport = this.xr_left_viewport;
    // gl.viewport(left_viewport.x, left_viewport.y, left_viewport.width, left_viewport.height);
    // gl.uniformMatrix4fv(pass_uniforms[0].loc, false, this.xr_left_projection_matrix);
    // gl.uniformMatrix4fv(pass_uniforms[1].loc, false, this.xr_left_transform_matrix);
    // gl.uniformMatrix4fv(pass_uniforms[2].loc, false, this.xr_left_invtransform_matrix);
    // this.ANGLE_instanced_arrays.drawElementsInstancedANGLE(gl.TRIANGLES, indices, gl.UNSIGNED_INT, 0, instances);
    // let right_viewport = this.xr_right_viewport;
    // gl.viewport(right_viewport.x, right_viewport.y, right_viewport.width, right_viewport.height);
    // gl.uniformMatrix4fv(pass_uniforms[0].loc, false, this.xr_right_projection_matrix);
    // gl.uniformMatrix4fv(pass_uniforms[1].loc, false, this.xr_right_transform_matrix);
    // gl.uniformMatrix4fv(pass_uniforms[2].loc, false, this.xr_right_invtransform_matrix);
    // this.ANGLE_instanced_arrays.drawElementsInstancedANGLE(gl.TRIANGLES, indices, gl.UNSIGNED_INT, 0, instances);
    // } else {
    for (let i = 0; i < passUniforms.length; i++) {
      const uni = passUniforms[i];
      uni.fn(this, uni.loc, uni.offset + passUniformsPtr);
    }
    this.ANGLEInstancedArrays.drawElementsInstancedANGLE(
      gl.TRIANGLES,
      indices,
      gl.UNSIGNED_INT,
      0,
      instances
    );
    // }
    this.OESVertexArrayObject.bindVertexArrayOES(null);
  }

  private allocTexture(
    textureId: number,
    width: number,
    height: number,
    dataPtr: number
  ): void {
    const gl = this.gl;
    const glTex = this.textures[textureId] || gl.createTexture();

    gl.bindTexture(gl.TEXTURE_2D, glTex);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

    const data = new Uint8Array(
      this.memory.buffer,
      dataPtr,
      width * height * 4
    );
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.RGBA,
      width,
      height,
      0,
      gl.RGBA,
      gl.UNSIGNED_BYTE,
      data
    );
    this.textures[textureId] = glTex as Texture;
  }

  private beginRenderTargets(
    passId: number,
    width: number,
    height: number
  ): void {
    const gl = this.gl;
    this.targetWidth = width;
    this.targetHeight = height;
    this.clearFlags = 0;
    // this.isMainCanvas = false;
    const glFramebuffer =
      this.framebuffers[passId] ||
      (this.framebuffers[passId] = assertNotNull(gl.createFramebuffer()));
    gl.bindFramebuffer(gl.FRAMEBUFFER, glFramebuffer);
  }

  private addColorTarget(
    textureId: number,
    initOnly: number,
    r: number,
    g: number,
    b: number,
    a: number
  ): void {
    // if use_default
    this.clearR = r;
    this.clearG = g;
    this.clearB = b;
    this.clearA = a;
    const gl = this.gl;

    const glTex =
      this.textures[textureId] ||
      (this.textures[textureId] = gl.createTexture() as Texture);

    // resize or create texture
    if (
      glTex.mpWidth != this.targetWidth ||
      glTex.mpHeight != this.targetHeight
    ) {
      gl.bindTexture(gl.TEXTURE_2D, glTex);
      this.clearFlags |= gl.COLOR_BUFFER_BIT;

      glTex.mpWidth = this.targetWidth;
      glTex.mpHeight = this.targetHeight;
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

      gl.texImage2D(
        gl.TEXTURE_2D,
        0,
        gl.RGBA,
        glTex.mpWidth,
        glTex.mpHeight,
        0,
        gl.RGBA,
        gl.UNSIGNED_BYTE,
        null
      );
    } else if (!initOnly) {
      this.clearFlags |= gl.COLOR_BUFFER_BIT;
    }

    gl.framebufferTexture2D(
      gl.FRAMEBUFFER,
      gl.COLOR_ATTACHMENT0,
      gl.TEXTURE_2D,
      glTex,
      0
    );
  }

  private setDepthTarget(
    textureId: number,
    initOnly: number,
    depth: number
  ): void {
    const gl = this.gl;
    this.clearDepth = depth;

    const glRenderBuffer =
      this.textures[textureId] ||
      (this.textures[textureId] = gl.createRenderbuffer() as Texture);

    if (
      glRenderBuffer.mpWidth != this.targetWidth ||
      glRenderBuffer.mpHeight != this.targetHeight
    ) {
      // Borrowed concept from https://webglfundamentals.org/webgl/lessons/webgl-render-to-texture.html
      gl.bindRenderbuffer(gl.RENDERBUFFER, glRenderBuffer);
      this.clearFlags |= gl.DEPTH_BUFFER_BIT;
      glRenderBuffer.mpWidth = this.targetWidth;
      glRenderBuffer.mpHeight = this.targetHeight;
      gl.renderbufferStorage(
        gl.RENDERBUFFER,
        gl.DEPTH_COMPONENT16,
        this.targetWidth,
        this.targetHeight
      );
    } else if (!initOnly) {
      this.clearFlags |= gl.DEPTH_BUFFER_BIT;
    }
    gl.framebufferRenderbuffer(
      gl.FRAMEBUFFER,
      gl.DEPTH_ATTACHMENT,
      gl.RENDERBUFFER,
      glRenderBuffer
    );
  }

  private endRenderTargets(): void {
    const gl = this.gl;

    // process the actual 'clear'
    gl.viewport(0, 0, this.targetWidth, this.targetHeight);

    // check if we need to clear color, and depth
    // clear it
    if (this.clearFlags) {
      gl.clearColor(this.clearR, this.clearG, this.clearB, this.clearA);
      gl.clearDepth(this.clearDepth);
      gl.clear(this.clearFlags);
    }
  }

  private setDefaultDepthAndBlendMode(): void {
    const gl = this.gl;
    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.LEQUAL);
    gl.blendEquationSeparate(gl.FUNC_ADD, gl.FUNC_ADD);
    gl.blendFuncSeparate(
      gl.ONE,
      gl.ONE_MINUS_SRC_ALPHA,
      gl.ONE,
      gl.ONE_MINUS_SRC_ALPHA
    );
    gl.enable(gl.BLEND);
  }

  private beginMainCanvas(
    r: number,
    g: number,
    b: number,
    a: number,
    depth: number
  ): void {
    const gl = this.gl;
    // this.isMainCanvas = true;
    // if (this.xrIsPresenting) {
    // let xr_webgllayer = this.xr_session.renderState.baseLayer;
    // this.gl.bindFramebuffer(gl.FRAMEBUFFER, xr_webgllayer.framebuffer);
    // gl.viewport(0, 0, xr_webgllayer.framebufferWidth, xr_webgllayer.framebufferHeight);
    // // quest 1 is 3648
    // // quest 2 is 4096
    // let left_view = this.xr_pose.views[0];
    // let right_view = this.xr_pose.views[1];
    // this.xr_left_viewport = xr_webgllayer.getViewport(left_view);
    // this.xr_right_viewport = xr_webgllayer.getViewport(right_view);
    // this.xr_left_projection_matrix = left_view.projectionMatrix;
    // this.xr_left_transform_matrix = left_view.transform.inverse.matrix;
    // this.xr_left_invtransform_matrix = left_view.transform.matrix;
    // this.xr_right_projection_matrix = right_view.projectionMatrix;
    // this.xr_right_transform_matrix = right_view.transform.inverse.matrix;
    // this.xr_right_camera_pos = right_view.transform.inverse.position;
    // this.xr_right_invtransform_matrix = right_view.transform.matrix;
    // } else {
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    gl.viewport(
      0,
      0,
      this.sizingData.width * this.sizingData.dpiFactor,
      this.sizingData.height * this.sizingData.dpiFactor
    );
    // }

    gl.clearColor(r, g, b, a);
    gl.clearDepth(depth);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
  }

  private uniformFnTable: Record<
    string,
    (zelf: WebGLRenderer, loc: WebGLUniformLocation | null, off: number) => void
  > = {
    float: function setFloat(zelf, loc, off) {
      const slot = off >> 2;
      zelf.gl.uniform1f(loc, zelf.basef32[slot]);
    },
    vec2: function setVec2(zelf, loc, off) {
      const slot = off >> 2;
      const basef32 = zelf.basef32;
      zelf.gl.uniform2f(loc, basef32[slot], basef32[slot + 1]);
    },
    vec3: function setVec3(zelf, loc, off) {
      const slot = off >> 2;
      const basef32 = zelf.basef32;
      zelf.gl.uniform3f(
        loc,
        basef32[slot],
        basef32[slot + 1],
        basef32[slot + 2]
      );
    },
    vec4: function setVec4(zelf, loc, off) {
      const slot = off >> 2;
      const basef32 = zelf.basef32;
      zelf.gl.uniform4f(
        loc,
        basef32[slot],
        basef32[slot + 1],
        basef32[slot + 2],
        basef32[slot + 3]
      );
    },
    mat2: function setMat2(zelf, loc, off) {
      zelf.gl.uniformMatrix2fv(
        loc,
        false,
        new Float32Array(zelf.memory.buffer, off, 4)
      );
    },
    mat3: function setMat3(zelf, loc, off) {
      zelf.gl.uniformMatrix3fv(
        loc,
        false,
        new Float32Array(zelf.memory.buffer, off, 9)
      );
    },
    mat4: function setMat4(zelf, loc, off) {
      const mat4 = new Float32Array(zelf.memory.buffer, off, 16);
      zelf.gl.uniformMatrix4fv(loc, false, mat4);
    },
  };

  // Array of function id's wasm can call on us; `zelf` is pointer to WebGLRenderer.
  // (It's not called `self` as to not overload https://developer.mozilla.org/en-US/docs/Web/API/Window/self)
  // Function names are suffixed with the index in the array, and annotated with
  // their name in cx_webgl.rs, for easier matching.
  private sendFnTable: ((zelf: this) => void | boolean)[] = [
    // end
    function end0(_zelf) {
      return true;
    },
    // compile_webgl_shader
    function compileWebGLShader1(zelf) {
      function parseShvarvec(): Uniform[] {
        const len = zelf.zerdeParser.parseU32();
        const vars: Uniform[] = [];
        for (let i = 0; i < len; i++) {
          vars.push({
            ty: zelf.zerdeParser.parseString() as UniformType,
            name: zelf.zerdeParser.parseString(),
          });
        }
        return vars;
      }

      const ash = {
        shaderId: zelf.zerdeParser.parseU32(),
        fragment: zelf.zerdeParser.parseString(),
        vertex: zelf.zerdeParser.parseString(),
        geometrySlots: zelf.zerdeParser.parseU32(),
        instanceSlots: zelf.zerdeParser.parseU32(),
        passUniforms: parseShvarvec(),
        viewUniforms: parseShvarvec(),
        drawUniforms: parseShvarvec(),
        userUniforms: parseShvarvec(),
        textureSlots: parseShvarvec(),
      };
      zelf.compileWebGLShader(ash);
    },
    // alloc_array_buffer
    function allocArrayBuffer2(zelf) {
      const arrayBufferId = zelf.zerdeParser.parseU32();
      const len = zelf.zerdeParser.parseU32();
      const pointer = zelf.zerdeParser.parseU32();
      const array = new Float32Array(zelf.memory.buffer, pointer, len);
      zelf.allocArrayBuffer(arrayBufferId, array);
    },
    // alloc_index_buffer
    function allocIndexBuffer3(zelf) {
      const indexBufferId = zelf.zerdeParser.parseU32();
      const len = zelf.zerdeParser.parseU32();
      const pointer = zelf.zerdeParser.parseU32();
      const array = new Uint32Array(zelf.memory.buffer, pointer, len);
      zelf.allocIndexBuffer(indexBufferId, array);
    },
    // alloc_vao
    function allocVao4(zelf) {
      const vaoId = zelf.zerdeParser.parseU32();
      const shaderId = zelf.zerdeParser.parseU32();
      const geomIbId = zelf.zerdeParser.parseU32();
      const geomVbId = zelf.zerdeParser.parseU32();
      const instVbId = zelf.zerdeParser.parseU32();
      zelf.allocVao(vaoId, shaderId, geomIbId, geomVbId, instVbId);
    },
    // draw_call
    function drawCall5(zelf) {
      const shaderId = zelf.zerdeParser.parseU32();
      const vaoId = zelf.zerdeParser.parseU32();
      const uniformsPassPtr = zelf.zerdeParser.parseU32();
      const uniformsViewPtr = zelf.zerdeParser.parseU32();
      const uniformsDrawPtr = zelf.zerdeParser.parseU32();
      const uniformsUserPtr = zelf.zerdeParser.parseU32();
      const textures = zelf.zerdeParser.parseU32();
      zelf.drawCall(
        shaderId,
        vaoId,
        uniformsPassPtr,
        uniformsViewPtr,
        uniformsDrawPtr,
        uniformsUserPtr,
        textures
      );
    },
    // update_texture_image2d
    function allocTexture6(zelf) {
      const textureId = zelf.zerdeParser.parseU32();
      const width = zelf.zerdeParser.parseU32();
      const height = zelf.zerdeParser.parseU32();
      const dataPtr = zelf.zerdeParser.parseU32();
      zelf.allocTexture(textureId, width, height, dataPtr);
    },
    // begin_render_targets
    function beginRenderTargets7(zelf) {
      const passId = zelf.zerdeParser.parseU32();
      const width = zelf.zerdeParser.parseU32();
      const height = zelf.zerdeParser.parseU32();
      zelf.beginRenderTargets(passId, width, height);
    },
    // add_color_target
    function addColorTarget8(zelf) {
      const textureId = zelf.zerdeParser.parseU32();
      const initOnly = zelf.zerdeParser.parseU32();
      const r = zelf.zerdeParser.parseF32();
      const g = zelf.zerdeParser.parseF32();
      const b = zelf.zerdeParser.parseF32();
      const a = zelf.zerdeParser.parseF32();
      zelf.addColorTarget(textureId, initOnly, r, g, b, a);
    },
    // set_depth_target
    function setDepthTarget9(zelf) {
      const textureId = zelf.zerdeParser.parseU32();
      const initOnly = zelf.zerdeParser.parseU32();
      const depth = zelf.zerdeParser.parseF32();
      zelf.setDepthTarget(textureId, initOnly, depth);
    },
    // end_render_targets
    function endRenderTargets10(zelf) {
      zelf.endRenderTargets();
    },
    // set_default_depth_and_blend_mode
    function setDefaultDepthAndBlendMode11(zelf) {
      zelf.setDefaultDepthAndBlendMode();
    },
    // begin_main_canvas
    function beginMainCanvas12(zelf) {
      const r = zelf.zerdeParser.parseF32();
      const g = zelf.zerdeParser.parseF32();
      const b = zelf.zerdeParser.parseF32();
      const a = zelf.zerdeParser.parseF32();
      const depth = zelf.zerdeParser.parseF32();
      zelf.beginMainCanvas(r, g, b, a, depth);
    },
  ];
}

const uniformSizeTable = {
  float: 1,
  vec2: 2,
  vec3: 3,
  vec4: 4,
  mat2: 4,
  mat3: 9,
  mat4: 16,
};

function addLineNumbersToString(code: string) {
  const lines = code.split("\n");
  let out = "";
  for (let i = 0; i < lines.length; i++) {
    out += i + 1 + ": " + lines[i] + "\n";
  }
  return out;
}
