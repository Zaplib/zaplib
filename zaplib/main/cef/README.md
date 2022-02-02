The Cef library for [Zaplib](https://github.com/Zaplib/zaplib).

This is early stage and experimental. For now, see the [repository](https://github.com/Zaplib/zaplib) for usage details. We will add better documentation over time.

## Publishing
When publishing this crate, make sure you're setting the `CEF_ROOT` env variable with the full path to Cef framework.

```
CEF_ROOT=`pwd`/zaplib/main/bind/cef-sys/deps/cef_binary_91.1.23+g04c8d56+chromium-91.0.4472.164_macosx64 cargo publish -p zaplib_cef
```
