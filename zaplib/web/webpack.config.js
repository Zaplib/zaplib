/* eslint-env node */
/* eslint-disable @typescript-eslint/no-var-requires */

const path = require("path");

// TODO(Paras): Export type definitions for our library builds, both for TypeScript
// and potentially Flow, using something like https://github.com/joarwilk/flowgen.

module.exports = (env, argv) => {
  return {
    entry: {
      /* eslint-disable camelcase */
      zaplib_runtime: "./zaplib_runtime.ts",
      zaplib_worker_runtime: "./zaplib_worker_runtime.ts",
      test_suite: "./test_suite.ts",
      /* eslint-enable camelcase */
    },
    output: {
      path: path.resolve(__dirname, "dist"),
      filename: "[name].js",
      library: {
        name: "zaplib",
        type: "umd",
      },
      clean: true,
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
