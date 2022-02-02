export function addDefaultStyles(): void {
  const style = document.createElement("style");
  style.innerHTML = `
  * {
    user-select: none;
  }
  html, body {
    overflow: hidden;
    background-color: #333;
  }
  body {
    margin: 0;
    position: fixed;
    width: 100%;
    height: 100%;
  }

  #zaplib_js_root {
    position: absolute; /* For z-index */
    z-index: 0; /* New stacking context */
    left: 0;
    right: 0;
    top: 0;
    bottom: 0;
    pointer-events: none;
  }`;
  document.body.appendChild(style);
}
