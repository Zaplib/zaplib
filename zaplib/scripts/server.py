#!/usr/bin/python2

# A small abstraction over SimpleHTTPServer so that
# we can set content headers correctly.

import SimpleHTTPServer
import SocketServer
import urllib
import ssl
import os

PORT = int(os.environ.get("PORT") or 3000)
WEBPACK_URL = "http://localhost:3001"
EXCLUDED_HEADERS = [
    "Cross-Origin-Embedder-Policy",
    "Cross-Origin-Opener-Policy",
    "Date"
]


class Handler(SimpleHTTPServer.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Access-Control-Allow-Origin", "*")
        SimpleHTTPServer.SimpleHTTPRequestHandler.end_headers(self)

    # Simple proxying for webpack requests. We need this because web workers
    # must be from the same origin.
    def do_GET(self):
        # Assume webpack requests always start with `dist`
        if self.path.startswith("/dist"):
            self.send_response(200)
            response = urllib.urlopen(WEBPACK_URL + self.path)

            for key, value in response.headers.items():
                key = "-".join([w.capitalize() for w in key.split("-")])
                if key not in EXCLUDED_HEADERS:
                    self.send_header(key, value)

            self.end_headers()
            self.copyfile(response, self.wfile)
        else:
            SimpleHTTPServer.SimpleHTTPRequestHandler.do_GET(self)


Handler.extensions_map['.wasm'] = 'application/wasm'

httpd = SocketServer.TCPServer(("0.0.0.0", PORT), Handler)

# To test with HTTPS, e.g. when testing a mobile device where you can't
# just use "localhost" and have to use an IP address (which will block
# usage of SharedArrayBuffer over regular HTTP), use this:
# 1. Generate a self-signed certificate using:
#    `openssl req -new -x509 -keyout localhost.pem -out localhost.pem -days 3650 -nodes`
# 2. Uncomment the code below, as set PORT=443.
# 3. Go to https://<your ip> and confirm that you want to visit the
#    website anyway, even though it's not a trusted certificate.
#
# httpd.socket = ssl.wrap_socket(httpd.socket,
#                                server_side=True,
#                                certfile='localhost.pem',
#                                ssl_version=ssl.PROTOCOL_TLS)

print("Serving at http://localhost:{}".format(PORT))
httpd.serve_forever()
