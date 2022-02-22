# Chromium Embedded Framework (CEF)

Zaplib apps that include JavaScript can also target native builds by using the [Chromium Embedded Framework](https://bitbucket.org/chromiumembedded/cef/src/master/#markdown-header-introduction). This is Zaplib's equivalent of Electron. CEF runs the JavaScript. The Rust and rendering code are compiled natively (i.e. no WebAssembly or WebGL), which is generally more performant. 

We do not recommend using this in production yet. It's currently mainly used to attach native debuggers and profilers. 

