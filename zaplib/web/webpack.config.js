/* eslint-env node */
/* eslint-disable @typescript-eslint/no-var-requires */

const webpack = require("webpack");
const path = require("path");
const { GitRevisionPlugin } = require("git-revision-webpack-plugin");

const gitRevisionPlugin = new GitRevisionPlugin();

let gitSha = "(no git sha found)";
try {
  gitSha = gitRevisionPlugin.commithash();
} catch (e) {
  console.log("no git sha found");
}

// TODO(Paras): Export type definitions for our library builds, both for TypeScript
// and potentially Flow, using something like https://github.com/joarwilk/flowgen.

const common = (env, argv) => {
  return {
    output: {
      path: path.resolve(__dirname, "dist"),
      filename:
        argv.mode === "production"
          ? "[name].production.js"
          : "[name].development.js",
      library: {
        name: "zaplib",
        type: "umd",
      },
      globalObject: "globalThis",
    },
    module: {
      rules: [
        {
          test: /\.tsx?$/,
          use: "ts-loader",
          exclude: /node_modules/,
        },
        {
          test: /\.css$/i,
          use: ["style-loader", "css-loader"],
        },
      ],
    },
    resolve: {
      modules: [path.resolve(__dirname, "."), "node_modules"],
      extensions: [".tsx", ".ts", ".js"],
    },
    devtool:
      argv.mode == "production" ? "source-map" : "eval-cheap-module-source-map",
    optimization: {
      // We shouldn't output non-entry chunks, but if we do, then this
      // helps in debugging.
      chunkIds: "named",
    },
    plugins: [
      new webpack.DefinePlugin({
        __GIT_SHA__: JSON.stringify(gitSha),
      }),
    ],
  };
};

const browserConfig = (env, argv) => {
  return {
    ...common(env, argv),
    entry: {
      /* eslint-disable camelcase */
      zaplib_runtime: "./zaplib_runtime.ts",
      zaplib_worker_runtime: "./zaplib_worker_runtime.ts",
      test_suite: "./test_suite/test_suite.ts",
      // for testing with Jest
      test_jest: "./jest/test_jest.ts",
      /* eslint-enable camelcase */
    },
  };
};

const nodeJsConfig = (env, argv) => {
  return {
    ...common(env, argv),
    target: "node",
    entry: {
      /* eslint-disable camelcase */
      // provides a set of polyfills for running in Node.js,
      // see zaplib.com/docs/existing_webapp.html#jest-integration for more details
      zaplib_nodejs_polyfill: "./zaplib_nodejs_polyfill.ts",
      /* eslint-enable camelcase */
    },
  };
};

module.exports = [browserConfig, nodeJsConfig];
