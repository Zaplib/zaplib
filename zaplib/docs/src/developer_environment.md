# Developer Environment


## VSCode

We recommend [VSCode](https://code.visualstudio.com/). We'll add guides for other editors/IDEs in the future.

1. [Install VSCode](https://code.visualstudio.com/download)
2. `VSCode > File > Open Workspace from File... >`[`zaplib/zaplib.code-workspace`](https://github.com/Zaplib/zaplib/blob/main/zaplib.code-workspace)
3. VSCode will prompt you to install our recommended extensions
4. We recommend NOT installing the official Rust extension since it conflicts with [`matklad.rust-analyzer`](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer). If you already have it installed, it's best to disable it.
5. Feel free to copy the settings from `zaplib.code-workspace` to your own projects!
6. Go to the "Run and Debug" tab in VSCode. In the dropdown at the top of that panel, you you should see a bunch of debug configs. They use [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb) as the debugger - so you can add breakpoints right in VSCode.

## Chrome debugging

To get Rust source maps when doing local development in [Chrome](https://www.google.com/chrome/):

1. Install the [WASM Debugginer Extension](https://goo.gle/wasm-debugging-extension).
2. `Chrome DevTools > Settings (gear-icon ⚙ in top-right corner) > Experiments > WebAssembly Debugging: Enable DWARF support` ([More info](https://developer.chrome.com/blog/wasm-debugging-2020/))

Note: these source maps read from hardcoded local file paths, so they'll only work on the computer that you've compiled on.

# TypeScript

Zaplib exports TypeScript types that should be picked up naturally. Check out the [TypeScript page of our docs](./typescript.md) for more information. 
