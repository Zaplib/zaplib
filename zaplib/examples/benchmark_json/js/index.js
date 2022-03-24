// Based on https://github.com/kostya/benchmarks/blob/1dd7deb29a813d1095e6062c25ad92bd81ce0273/json/test.js

'use strict';

function calc(text) {
  const jobj = JSON.parse(text);

  const coordinates = jobj['coordinates'];
  const len = coordinates.length;
  let x = 0;
  let y = 0;
  let z = 0;

  for (let i = 0; i < coordinates.length; i++) {
    const coord = coordinates[i];
    x += coord['x'];
    y += coord['y'];
    z += coord['z'];
  }

  return {
    x: x / len,
    y: y / len,
    z: z / len
  };
}

fetch('../data.json').then(response => response.text()).then(text => {
  const start = performance.now()
  const results = calc(text);
  const end = performance.now();
  
  console.log(`Time: ${end - start} ms`);
});

