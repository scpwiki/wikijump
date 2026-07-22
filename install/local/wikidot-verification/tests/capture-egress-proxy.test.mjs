import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import net from "node:net";
import path from "node:path";
import { createRequire } from "node:module";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  guardedPipeline,
  isBlockedAddress,
  resolvePinned,
  startCaptureEgressProxy,
} from "../src/capture-egress-proxy.mjs";

const repoRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../../..",
);

test("guarded pipeline reports a synchronous closed-stream failure instead of throwing", () => {
  const failure = new Error("destination already closed");
  let observed = null;
  assert.doesNotThrow(() => guardedPipeline({}, {}, (error) => {
    observed = error;
  }, () => {
    throw failure;
  }));
  assert.equal(observed, failure);
});

async function listen(handler) {
  const server = http.createServer(handler);
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  return { server, port: server.address().port };
}

function close(server) {
  server.closeAllConnections?.();
  return new Promise((resolve) => server.close(resolve));
}

function proxyRequest(proxyUrl, target, { method = "GET", body = "" } = {}) {
  const proxy = new URL(proxyUrl);
  return new Promise((resolve, reject) => {
    const request = http.request(
      { host: proxy.hostname, port: proxy.port, method, path: target },
      (response) => {
        let data = "";
        response.setEncoding("utf8");
        response.on("data", (chunk) => {
          data += chunk;
        });
        response.on("end", () =>
          resolve({ status: response.statusCode, body: data }),
        );
      },
    );
    request.on("error", reject);
    request.end(body);
  });
}

test("address policy rejects private, loopback, link-local, metadata, and invalid addresses", () => {
  for (const value of [
    "127.0.0.1",
    "10.0.0.1",
    "172.16.0.1",
    "192.168.1.1",
    "169.254.169.254",
    "100.100.100.200",
    "::1",
    "fd00::1",
    "fe80::1",
    "not-an-ip",
  ]) {
    assert.equal(isBlockedAddress(value), true, value);
  }
  assert.equal(isBlockedAddress("93.184.216.34"), false);
  assert.equal(isBlockedAddress("2606:2800:220:1:248:1893:25c8:1946"), false);
});

test("proxy pins one resolution and forwards method/body only to an exact allowed local origin", async () => {
  const upstream = await listen((request, response) => {
    let body = "";
    request.on("data", (chunk) => {
      body += chunk;
    });
    request.on("end", () => response.end(`${request.method}:${body}`));
  });
  let lookups = 0;
  const proxy = await startCaptureEgressProxy({
    allowedLocalOrigins: [`http://fixture.test:${upstream.port}`],
    lookup: async () => {
      lookups += 1;
      return [{ address: "127.0.0.1" }];
    },
  });
  try {
    assert.deepEqual(
      await proxyRequest(
        proxy.url,
        `http://fixture.test:${upstream.port}/post`,
        { method: "POST", body: "payload" },
      ),
      { status: 200, body: "POST:payload" },
    );
    assert.equal(lookups, 1);
    assert.equal(
      (
        await proxyRequest(
          proxy.url,
          `http://fixture.test:${upstream.port + 1}/`,
        )
      ).status,
      403,
    );
  } finally {
    await proxy.close();
    await close(upstream.server);
  }
});

test("a DNS rebinding answer is revalidated and denied before connection", async () => {
  let lookupCount = 0;
  const lookup = async () => [
    { address: ++lookupCount === 1 ? "93.184.216.34" : "127.0.0.1" },
  ];
  assert.equal(
    await resolvePinned("rebind.test", 80, { lookup }),
    "93.184.216.34",
  );
  await assert.rejects(
    resolvePinned("rebind.test", 80, { lookup }),
    /not publicly routable/u,
  );
  assert.equal(lookupCount, 2);
});

test("an aborted streamed response does not crash the proxy", async () => {
  const upstream = await listen((request, response) => {
    if (request.url === "/ok") return response.end("OK");
    response.write("first");
    setTimeout(() => response.end("second"), 25);
  });
  const proxy = await startCaptureEgressProxy({
    allowedLocalOrigins: [`http://fixture.test:${upstream.port}`],
    lookup: async () => [{ address: "127.0.0.1" }],
  });
  try {
    const proxyUrl = new URL(proxy.url);
    await new Promise((resolve, reject) => {
      const request = http.get(
        {
          host: proxyUrl.hostname,
          port: proxyUrl.port,
          path: `http://fixture.test:${upstream.port}/stream`,
        },
        (response) => {
          response.once("data", () => {
            response.destroy();
            resolve();
          });
          response.once("error", reject);
        },
      );
      request.once("error", reject);
    });
    await new Promise((resolve) => setTimeout(resolve, 50));
    assert.deepEqual(
      await proxyRequest(proxy.url, `http://fixture.test:${upstream.port}/ok`),
      { status: 200, body: "OK" },
    );
  } finally {
    await proxy.close();
    await close(upstream.server);
  }
});

test("CONNECT rejects a private destination unless its exact origin is allowed", async () => {
  const proxy = await startCaptureEgressProxy();
  const address = new URL(proxy.url);
  try {
    const reply = await new Promise((resolve, reject) => {
      const socket = net.connect(Number(address.port), address.hostname, () =>
        socket.write(
          "CONNECT 127.0.0.1:443 HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
        ),
      );
      socket.once("data", (data) => {
        resolve(data.toString());
        socket.destroy();
      });
      socket.once("error", reject);
    });
    assert.match(reply, /^HTTP\/1\.1 403/u);
  } finally {
    await proxy.close();
  }
});

test("an aborted CONNECT tunnel does not crash the proxy", async () => {
  const upstream = net.createServer((socket) => socket.on("error", () => {}));
  await new Promise((resolve) => upstream.listen(0, "127.0.0.1", resolve));
  const upstreamPort = upstream.address().port;
  const proxy = await startCaptureEgressProxy({
    allowedLocalOrigins: [`https://fixture.test:${upstreamPort}`],
    lookup: async () => [{ address: "127.0.0.1" }],
  });
  const proxyUrl = new URL(proxy.url);
  try {
    await new Promise((resolve, reject) => {
      const socket = net.connect(Number(proxyUrl.port), proxyUrl.hostname, () =>
        socket.write(
          `CONNECT fixture.test:${upstreamPort} HTTP/1.1\r\nHost: fixture.test\r\n\r\n`,
        ),
      );
      socket.once("data", (data) => {
        assert.match(data.toString(), /^HTTP\/1\.1 200/u);
        if (typeof socket.resetAndDestroy === "function")
          socket.resetAndDestroy();
        else socket.destroy();
        resolve();
      });
      socket.once("error", reject);
    });
    await new Promise((resolve) => setTimeout(resolve, 50));

    const reply = await new Promise((resolve, reject) => {
      const socket = net.connect(Number(proxyUrl.port), proxyUrl.hostname, () =>
        socket.write(
          `CONNECT fixture.test:${upstreamPort} HTTP/1.1\r\nHost: fixture.test\r\n\r\n`,
        ),
      );
      socket.once("data", (data) => {
        resolve(data.toString());
        socket.destroy();
      });
      socket.once("error", reject);
    });
    assert.match(reply, /^HTTP\/1\.1 200/u);
  } finally {
    await proxy.close();
    await new Promise((resolve) => upstream.close(resolve));
  }
});

test("an early CONNECT reset during DNS resolution does not crash the proxy or dial upstream", async () => {
  let upstreamConnections = 0;
  const upstream = net.createServer((socket) => {
    upstreamConnections += 1;
    socket.on("error", () => {});
  });
  await new Promise((resolve) => upstream.listen(0, "127.0.0.1", resolve));
  const upstreamPort = upstream.address().port;
  let releaseFirstLookup;
  const firstLookupStarted = Promise.withResolvers();
  let lookupCount = 0;
  const proxy = await startCaptureEgressProxy({
    allowedLocalOrigins: [`https://fixture.test:${upstreamPort}`],
    lookup: async () => {
      lookupCount += 1;
      if (lookupCount === 1) {
        firstLookupStarted.resolve();
        await new Promise((resolve) => {
          releaseFirstLookup = resolve;
        });
      }
      return [{ address: "127.0.0.1" }];
    },
  });
  const proxyUrl = new URL(proxy.url);
  try {
    const socket = net.connect(Number(proxyUrl.port), proxyUrl.hostname, () =>
      socket.write(
        `CONNECT fixture.test:${upstreamPort} HTTP/1.1\r\nHost: fixture.test\r\n\r\n`,
      ),
    );
    socket.on("error", () => {});
    await firstLookupStarted.promise;
    if (typeof socket.resetAndDestroy === "function") socket.resetAndDestroy();
    else socket.destroy();
    await new Promise((resolve) => setTimeout(resolve, 25));
    releaseFirstLookup();
    await new Promise((resolve) => setTimeout(resolve, 25));
    assert.equal(upstreamConnections, 0);

    const reply = await new Promise((resolve, reject) => {
      const next = net.connect(Number(proxyUrl.port), proxyUrl.hostname, () =>
        next.write(
          `CONNECT fixture.test:${upstreamPort} HTTP/1.1\r\nHost: fixture.test\r\n\r\n`,
        ),
      );
      next.once("data", (data) => {
        resolve(data.toString());
        next.destroy();
      });
      next.once("error", reject);
    });
    assert.match(reply, /^HTTP\/1\.1 200/u);
  } finally {
    releaseFirstLookup?.();
    await proxy.close();
    await new Promise((resolve) => upstream.close(resolve));
  }
});

test(
  "real Chromium allows same-origin iframe/fetch/POST and blocks redirect to another local origin",
  { timeout: 30_000 },
  async (t) => {
    const chromePath = "/usr/bin/google-chrome";
    if (!fs.existsSync(chromePath)) {
      t.skip("system Chrome is not installed");
      return;
    }
    let chromium;
    try {
      const require = createRequire(
        path.join(repoRoot, "framerail/package.json"),
      );
      ({ chromium } = require("@playwright/test"));
    } catch {
      t.skip("Playwright is not installed");
      return;
    }
    const internal = await listen((_request, response) =>
      response.end("INTERNAL SECRET"),
    );
    const fixture = await listen((request, response) => {
      if (request.url === "/frame") return response.end("FRAME OK");
      if (request.url === "/api" && request.method === "POST")
        return response.end("POST OK");
      if (request.url === "/redirect") {
        response.writeHead(302, {
          location: `http://internal.test:${internal.port}/secret`,
        });
        return response.end();
      }
      response.end(`<iframe src="/frame"></iframe><script>
      Promise.all([
        fetch('/api', {method:'POST', body:'x'}).then(r=>r.text()),
        fetch('/redirect').then(r=>r.text()).catch(()=> 'REDIRECT BLOCKED')
      ]).then(([post, blocked]) => document.body.dataset.result = post + '|' + blocked);
    </script>`);
    });
    const lookup = async (hostname) => {
      if (hostname === "fixture.test" || hostname === "internal.test")
        return [{ address: "127.0.0.1" }];
      return [];
    };
    const proxy = await startCaptureEgressProxy({
      allowedLocalOrigins: [`http://fixture.test:${fixture.port}`],
      lookup,
    });
    const browser = await chromium.launch({
      executablePath: chromePath,
      headless: true,
      proxy: { server: proxy.url, bypass: "<-loopback>" },
    });
    try {
      const page = await browser.newPage();
      await page.goto(`http://fixture.test:${fixture.port}/`);
      await page.waitForFunction(() => document.body.dataset.result);
      assert.equal(
        await page.locator("iframe").contentFrame().locator("body").innerText(),
        "FRAME OK",
      );
      assert.equal(
        await page.locator("body").getAttribute("data-result"),
        "POST OK|REDIRECT BLOCKED",
      );
    } finally {
      await browser.close();
      await proxy.close();
      await close(fixture.server);
      await close(internal.server);
    }
  },
);
