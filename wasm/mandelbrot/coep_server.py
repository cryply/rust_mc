#!/usr/bin/env python3
import http.server
import socketserver
import argparse

class COEPHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        super().end_headers()

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--port', '-p', type=int, default=8080)
    parser.add_argument('--bind', '-b', default='127.0.0.1')
    args = parser.parse_args()
    
    with socketserver.TCPServer((args.bind, args.port), COEPHandler) as httpd:
        print(f'Serving at http://{args.bind}:{args.port}')
        httpd.serve_forever()
