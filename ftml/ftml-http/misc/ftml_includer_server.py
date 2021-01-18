#!/usr/bin/env python3

#
# misc/ftml_includer_server.py
#
# ftml - Library to parse Wikidot text
# Copyright (C) 2019-2021 Ammon Smith
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with this program. If not, see <http://www.gnu.org/licenses/>.
#

"""
Sample server to respond to ftml-http's requests for pages.

Has a few pages which are responded to with special content:

    * `component:theme`               - Some CSS
    * `theme:black-highlighter-theme` - Some other CSS
    * contains `missing`              - Will always be absent
    * contains `echo`                 - Prints out the page reference
"""

import json
from http.server import HTTPServer, BaseHTTPRequestHandler

PORT = 8000

SIGMA_PAGE = """\
[[module CSS]]
/* SOME NICE BOXES */
div.sexy-box {
    background: #eee;
    border: 1px solid #ccc;
    padding: 0 10px 12px;
    margin: 7px 4px 12px;
    overflow: hidden;
}
[[/module]]
"""

BHL_PAGE = """\
[[module CSS]]
:root {
    --logo-image: url("https://nu-scptheme.github.io/Black-Highlighter/img/logo.svg");
    --header-title: "SCP Foundation";
    --header-subtitle: "SECURE, CONTAIN, PROTECT";
}
[[/module]]
"""

def generate_page(site, page):
    if page == "component:theme":
        return SIGMA_PAGE
    elif page == "theme:black-highlighter-theme":
        return BHL_PAGE
    elif "missing" in page:
        return None
    elif "echo" in page:
        return f"Site: {site}, Page: {page}"
    else:
        return "Some page content here!"


def generate_pages(page_refs):
    print(f"Requested pages: {page_refs}")

    page_data = []
    for page_ref in page_refs:
        site = page_ref["site"]
        page = page_ref["page"]

        content = generate_page(site, page)
        if content is not None:
            page_data.append({
                "page": page_ref,
                "content": content,
            })

    return page_data

class RequestHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        # Read and process request
        length = int(self.headers.get('Content-Length'))
        request = self.rfile.read(length)

        page_refs = json.loads(request)
        page_data = generate_pages(page_refs)
        print(f"Sending response: {page_data}")

        # Write headers
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()

        # Write response
        response = json.dumps(page_data)
        self.wfile.write(response.encode("utf-8"))


if __name__ == "__main__":
    print(f"Starting HTTP server on port {PORT}")
    httpd = HTTPServer(("", PORT), RequestHandler)

    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        httpd.server_close()
