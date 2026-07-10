import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { cp, mkdtemp, readFile, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const caddyRoot = fileURLToPath(new URL("..", import.meta.url));
const deployHostFilter =
  '.params.deploy_host | strings | select(test("^[A-Za-z0-9.-]+:[0-9]+$"))';

test("production image build substitutes the static deploy host exactly once", async (t) => {
  const temporaryRoot = await mkdtemp(
    path.join(os.tmpdir(), "wikijump-prod-caddy-"),
  );
  t.after(() => rm(temporaryRoot, { recursive: true, force: true }));
  const renderedPath = path.join(temporaryRoot, "Caddyfile");
  await cp(path.join(caddyRoot, "Caddyfile"), renderedPath);

  const result = spawnSync(
    "sh",
    [
      "-eu",
      "-c",
      `deploy_host="$(jq -er '${deployHostFilter}' "$1")"; sed -i "s/<<DEPLOY_HOST>>/${"${deploy_host}"}/" "$2"`,
      "sh",
      path.join(caddyRoot, "request.json"),
      renderedPath,
    ],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 0, result.stderr);

  const request = JSON.parse(
    await readFile(path.join(caddyRoot, "request.json"), "utf8"),
  );
  const rendered = await readFile(renderedPath, "utf8");
  assert.equal(request.params.deploy_host, "host.docker.internal:9120");
  assert.doesNotMatch(rendered, /<<DEPLOY_HOST>>/u);
  assert.match(rendered, /reverse_proxy host\.docker\.internal:9120/u);

  const validation = spawnSync(
    "docker",
    [
      "run",
      "--rm",
      "-v",
      `${renderedPath}:/etc/caddy/Caddyfile:ro`,
      "caddy:alpine",
      "caddy",
      "validate",
      "--config",
      "/etc/caddy/Caddyfile",
      "--adapter",
      "caddyfile",
    ],
    { encoding: "utf8" },
  );
  assert.equal(validation.status, 0, validation.stderr);
  assert.match(
    `${validation.stdout}\n${validation.stderr}`,
    /Valid configuration/u,
  );
});

test("Dockerfile substitution consumes only tracked static inputs", async () => {
  const dockerfile = await readFile(path.join(caddyRoot, "Dockerfile"), "utf8");
  const caddyfile = await readFile(path.join(caddyRoot, "Caddyfile"), "utf8");
  const request = JSON.parse(
    await readFile(path.join(caddyRoot, "request.json"), "utf8"),
  );

  assert.equal(caddyfile.match(/<<DEPLOY_HOST>>/gu)?.length, 1);
  assert.equal(typeof request.params.deploy_host, "string");
  assert.match(request.params.deploy_host, /^[A-Za-z0-9.-]+:[0-9]+$/u);
  assert.match(dockerfile, /jq -er/u);
  assert.match(dockerfile, /sed -i "s\/<<DEPLOY_HOST>>\/\$\{deploy_host\}\//u);
  assert.match(dockerfile, /! grep -q '<</u);
  assert.match(dockerfile, /caddy validate --config \/etc\/caddy\/Caddyfile/u);
});

test("deploy host extraction fails closed on malformed values", () => {
  for (const value of [
    null,
    "",
    "host",
    "host:port",
    "host:9120/path",
    "host:9120&injected",
    "host:9120\nextra",
  ]) {
    const result = spawnSync("jq", ["-er", deployHostFilter], {
      input: JSON.stringify({ params: { deploy_host: value } }),
      encoding: "utf8",
    });
    assert.notEqual(result.status, 0, JSON.stringify(value));
  }
});
