addEventListener('message', e => {
  postMessage("dummy:" + e.data);
});
