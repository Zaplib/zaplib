/* eslint-env node */

"use strict";

if (process.env.NODE_ENV === "production") {
  module.exports = require("./zaplib_nodejs_polyfill.production.js");
} else {
  module.exports = require("./zaplib_nodejs_polyfill.development.js");
}
