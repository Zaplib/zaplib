/* eslint-env node */

"use strict";

if (process.env.NODE_ENV === "production") {
  module.exports = require("./zaplib_worker_runtime.production.js");
} else {
  module.exports = require("./zaplib_worker_runtime.development.js");
}
