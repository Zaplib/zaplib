The file node.js was venodred from `cjs/node.js` from web-worker repo as of [`29fef97757`](https://github.com/developit/web-worker/tree/29fef9775702c91887d3d8733e595edf1a188f31) commit using the patch after running the `yarn prepare` command:

```diff
diff --git a/node.js b/node.js
index 9d88718..90fe54f 100644
--- a/node.js
+++ b/node.js
@@ -211,6 +211,8 @@ function evaluateDataUrl(url, name) {
 function parseDataUrl(url) {
        let [m, type, encoding, data] = url.match(/^data: *([^;,]*)(?: *; *([^,]*))? *,(.*)$/) || [];
        if (!m) throw Error('Invalid Data URL.');
+
+       data = decodeURIComponent(data);
        if (encoding) switch (encoding.toLowerCase()) {
                case 'base64':
                        data = Buffer.from(data, 'base64').toString();
```
