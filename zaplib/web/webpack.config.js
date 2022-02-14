/* eslint-env node */
/* eslint-disable @typescript-eslint/no-var-requires */

const path = require("path");

const FilterWarningsPlugin = require("webpack-filter-warnings-plugin");

// TODO(Paras): Export type definitions for our library builds, both for TypeScript
// and potentially Flow, using something like https://github.com/joarwilk/flowgen.

const browserConfig = (env, argv) => {
  return {
    entry: {
      /* eslint-disable camelcase */
      zaplib_runtime: "./zaplib_runtime.ts",
      zaplib_worker_runtime: "./zaplib_worker_runtime.ts",
      test_suite: "./test_suite.ts",
      // for testing with Jest
      test_jest: "./jest/test_jest.ts",
      /* eslint-enable camelcase */
    },
    output: {
      path: path.resolve(__dirname, "dist"),
      filename: "[name].js",
      library: {
        name: "zaplib",
        type: "umd",
      },
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
      extensions: [".tsx", ".ts", ".js"],
    },
    devtool:
      argv.mode == "production" ? "source-map" : "eval-cheap-module-source-map",
    optimization: {
      // We shouldn't output non-entry chunks, but if we do, then this
      // helps in debugging.
      chunkIds: "named",
    },
  };
};

const nodeJsConfig = (env, argv) => {
  return {
    target: "node",
    entry: {
      /* eslint-disable camelcase */
      // provides a set of polyfills for running in Node.js,
      // see zaplib.com/docs/basic_tooling.html#jest for more details
      zaplib_nodejs_polyfill: "./zaplib_nodejs_polyfill.ts",
      /* eslint-enable camelcase */
    },
    output: {
      path: path.resolve(__dirname, "dist"),
      filename: "[name].js",
      library: {
        name: "zaplib",
        type: "umd",
      },
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
      new FilterWarningsPlugin({
        // Suppress warnings coming from ./vendor/web-worker/node.js as this is expected
        // We are dynamically importing worker scripts
        exclude:
          /Critical dependency: the request of a dependency is an expression/,
      }),
    ],
  };
};

module.exports = [browserConfig, nodeJsConfig];
