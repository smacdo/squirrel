import http
from http.server import HTTPServer
import logging


class HttpServer:
    stopped = False

    def __init__(self, port: int, directory: str):
        self.port = port
        self.directory = directory

    def run(self):
        directory_to_server = self.directory

        class BuildHttpHandler(http.server.SimpleHTTPRequestHandler):
            def __init__(self, *args, **kwargs):
                super().__init__(*args, directory=directory_to_server, **kwargs)

        logging.info(f"Running HTTP server at http://localhost:{self.port}")

        server_address = ("", self.port)
        httpd = HTTPServer(server_address, BuildHttpHandler)
        httpd.timeout = 1.0

        while not self.stopped:
            httpd.handle_request()

        logging.debug("http server thread stopped")

    def stop(self):
        logging.debug("http server got stop request")
        self.stopped = True
