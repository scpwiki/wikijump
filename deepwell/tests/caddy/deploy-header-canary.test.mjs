import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import test from "node:test";

const listen = (server) =>
  new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const close = (server) =>
  new Promise((resolve, reject) => {
    server.close((error) => (error ? reject(error) : resolve()));
  });

test("deploy proxy strips mixed-case and repeated Wikijump trust headers", async (t) => {
  const echo = http.createServer((request, response) => {
    response.setHeader("content-type", "application/json");
    response.end(JSON.stringify(request.rawHeaders));
  });
  await listen(echo);
  t.after(() => close(echo));
  const echoPort = echo.address().port;

  const reservation = http.createServer();
  await listen(reservation);
  const caddyPort = reservation.address().port;
  await close(reservation);

  const temporaryRoot = await mkdtemp(
    path.join(os.tmpdir(), "wikijump-deploy-header-"),
  );
  t.after(() => rm(temporaryRoot, { recursive: true, force: true }));
  const configPath = path.join(temporaryRoot, "Caddyfile");
  await writeFile(
    configPath,
    `
(strip_headers) {
  request_header -X-Wikijump-*
}

:${caddyPort} {
  import strip_headers
  reverse_proxy 127.0.0.1:${echoPort}
}
`,
  );

  const container = `wikijump-header-canary-${process.pid}`;
  const started = spawnSync(
    "docker",
    [
      "run",
      "--detach",
      "--rm",
      "--network",
      "host",
      "--name",
      container,
      "-v",
      `${configPath}:/etc/caddy/Caddyfile:ro`,
      "caddy:alpine",
      "caddy",
      "run",
      "--config",
      "/etc/caddy/Caddyfile",
      "--adapter",
      "caddyfile",
    ],
    { encoding: "utf8" },
  );
  assert.equal(started.status, 0, started.stderr);
  t.after(() => spawnSync("docker", ["rm", "--force", container]));

  let response;
  let lastError;
  for (let attempt = 0; attempt < 30; attempt += 1) {
    try {
      response = await new Promise((resolve, reject) => {
        const request = http.request(
          {
            hostname: "127.0.0.1",
            port: caddyPort,
            headers: [
              "Host",
              `127.0.0.1:${caddyPort}`,
              "X-Wikijump-Site-Id",
              "attacker-one",
              "x-WIKIJUMP-site-id",
              "attacker-two",
              "X-Wikijump-Site-Slug",
              "attacker-site",
              "X-Wikijump-Session-Token",
              "attacker-token",
              "X-Wikijump-Target-Server",
              "files",
              "X-Wikijump-Basic-Error",
              "1",
              "X-Wikijump-",
              "empty-suffix",
              "X-Wikijumpx-Canary",
              "similar-but-untrusted",
              "X-Ordinary-Canary",
              "preserved",
            ],
          },
          (incoming) => {
            let body = "";
            incoming.setEncoding("utf8");
            incoming.on("data", (chunk) => {
              body += chunk;
            });
            incoming.on("end", () => {
              if (incoming.statusCode !== 200) {
                reject(
                  new Error(`Caddy returned ${incoming.statusCode}: ${body}`),
                );
                return;
              }
              resolve(JSON.parse(body));
            });
          },
        );
        request.on("error", reject);
        request.end();
      });
      break;
    } catch (error) {
      lastError = error;
      await new Promise((resolve) => setTimeout(resolve, 50));
    }
  }

  assert.ok(response, `Caddy did not become ready: ${lastError}`);
  const names = response
    .filter((_, index) => index % 2 === 0)
    .map((name) => name.toLowerCase());
  assert.equal(
    names.some((name) => name.startsWith("x-wikijump-")),
    false,
  );
  assert.equal(names.includes("x-wikijumpx-canary"), true);
  assert.equal(names.includes("x-ordinary-canary"), true);
  assert.equal(response.includes("similar-but-untrusted"), true);
  assert.equal(response.includes("preserved"), true);
});
