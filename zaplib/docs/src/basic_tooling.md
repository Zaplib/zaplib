# Basic Tooling

Now that you're able to [run some examples](./getting_started.md), lets set up your development environment.

## Editor: VSCode

* We currently recommend using [VSCode](https://code.visualstudio.com/). In the future we'll add guides for other editors/IDEs.
* After installing VSCode, open up `workspace.code-workspace` in the root of the repo. VSCode will prompt you to install our recommended extensions.
* We recommend NOT installing the official Rust extension since it conflicts with [`matklad.rust-analyzer`](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer). If you already have it installed, it's best to disable it.
* Feel free to copy the settings from `workspace.code-workspace` to your own projects!

If you go to the "Run and Debug" tab in VSCode, you should see a bunch of preconfigured run profiles at the top of the screen (from [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)).

## Chrome debugging

To get Rust source maps when doing local development in [Chrome](https://www.google.com/chrome/):
* Install [this extension](https://goo.gle/wasm-debugging-extension).
* Open Chrome DevTools, click the gear (⚙) icon in the top right corner of DevTools pane, go to the Experiments panel and tick **WebAssembly Debugging: Enable DWARF support**. (See also [this article](https://developer.chrome.com/blog/wasm-debugging-2020/)).

Note that these source maps read from hardcoded local file paths, so they'll only work on the computer that you've compiled on.
