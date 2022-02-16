#!/usr/bin/python2

import SimpleHTTPServer
import SocketServer

PORT = 3000
class Handler(SimpleHTTPServer.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Access-Control-Allow-Origin", "*")
        SimpleHTTPServer.SimpleHTTPRequestHandler.end_headers(self)

Handler.extensions_map['.wasm'] = 'application/wasm'
httpd = SocketServer.TCPServer(("", PORT), Handler)
httpd.serve_forever()
