# Webpack Integration

Zaplib supports building using Webpack, and will automatically use development or production builds based on environment.

## Installing

Add Zaplib to your package dependencies using `npm install zaplib` or `yarn add zaplib` from your project root. This should add Zaplib to your `package.json`.

## Usage:
 - For the main thread runtime, use: `import zaplib from 'zaplib';`.
 - For the worker runtime, use: `import * as zaplib from 'zaplib/dist/zaplib_worker_runtime';`.

