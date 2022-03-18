# Zapium

Zapium is the native, cross-platform Zaplib runtime. It converts Zaplib web apps to desktop apps, _where the Rust code runs natively, not via WebAssembly_.

This is Zaplib's equivalent of Electron -- for an extra speed boost. Your JavaScript is run via the [Chromium Embedded Framework](https://bitbucket.org/chromiumembedded/cef/src/master/#markdown-header-introduction), and your Zaplib Rust & shader code is compiled natively (i.e. no WebAssembly or WebGL).

Currently the Zapium code is open-source licensed just like all the other code in our repo, but we'll likely change Zapium's license in the near future to be commercially licensed and source-available.

Contact us if you would like to use this in production.
