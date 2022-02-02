Generates bindings for CEF on the fly. Requires an extracted CEF "minimal" directory to be present in `deps/`, which typically gets installed through `scripts/install_deps_macos.sh`. See `build.rs` for more details.

## Notes on old version (Chrome 91)

We're currently stuck on CEF/Chromium 91, because sending messages in single-process mode gets broken after that. In versions after that, we sometimes seem to get stuck with a seemingly unattached frame, which causes all messages to get stuck in this queue: https://github.com/chromiumembedded/cef/blob/a7bbd8a62bfc91b0d53eeef8d07b64a5ed719a5f/libcef/browser/frame_host_impl.cc#L559 We filed a bug report here: https://bitbucket.org/chromiumembedded/cef/issues/3191/renderer-stopped-getting and this might be related too: https://magpcss.org/ceforum/viewtopic.php?f=6&t=18659 TODO(JP): Dig deeper into what is happening here and file a more specific bug report with CEF or Chromium, depending on where the root cause is.

## Debugging

To debug with symbols, run `scripts/install_deps_macos_cef_symbols.sh`. To view actual CEF source code during debugging, put the CEF git repo at `~/cef` and make sure to check out the matching commit. (It's not adviced to symlink the CEF source code, as VSCode+lldb doesn't always properly follow symlinks.) For example: `git clone --branch 4472 https://github.com/chromiumembedded/cef.git ~/cef`. Then, run lldb with a source map like this:

```
settings set target.source-map /Users/spotify-buildagent/buildAgent/work/CEF3_git/chromium/src/cef ~/cef
```

This path was obtained by running `drawfdump` and looking at the paths; e.g. by running

```
dwarfdump zaplib/main/bind/cef-sys/deps/cef_binary_93.1.11+g9e254fa+chromium-93.0.4577.63_macosx64_minimal/Release/Chromium\ Embedded\ Framework.framework/Chromium\ Embedded\ Framework.dSYM
```

When using VSCode, you can use the [CodeLLDB extension](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb) with a configuration like this:

```js
{
    "type": "lldb",
    "request": "attach",
    "name": "Attach to running 'test_suite'",
    "program": "target/debug/test_suite",
    "sourceMap": {
        "/Users/spotify-buildagent/buildAgent/work/CEF3_git/chromium/src/cef": "${env:HOME}/cef",
    },
    "sourceLanguages": ["cpp", "rust"]
},
```

TODO(JP): It might be nice to also be able to show the Chrome source code; see e.g. https://bitbucket.org/chromiumembedded/cef/wiki/MasterBuildQuickStart

## Publishing
When publishing this crate, make sure you're setting the `CEF_ROOT` env variable with the full path to Cef framework.

```
CEF_ROOT=`pwd`/zaplib/main/bind/cef-sys/deps/cef_binary_91.1.23+g04c8d56+chromium-91.0.4472.164_macosx64 cargo publish -p zaplib_cef_sys
```
