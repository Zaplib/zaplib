# TypeScript

Zaplib was written in TypeScript and exports TypeScript types.

You may need to manually import the type declarations if you are directly using `zaplib.production.js` or `zaplib.development.js` -- which is how all the examples in these docs are setup to keep them simple (no webpack or other bundling). You can manually import the types via a [TypeScript reference](https://www.typescriptlang.org/docs/handbook/triple-slash-directives.html#-reference-types-):

```ts
/// <reference types="zaplib_runtime" />
```
