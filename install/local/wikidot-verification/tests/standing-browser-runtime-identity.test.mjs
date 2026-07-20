import assert from "node:assert/strict";
import test from "node:test";

import {
  assertStableCandidateRuntimeIdentity,
  effectiveRuntimeServicesSha256,
  observeCandidateRuntimeIdentity,
  validateCandidateRuntimeObservation,
} from "../src/standing-browser-runtime-identity.mjs";

const hash = (character) => character.repeat(64);
const git = (character) => character.repeat(40);

function identity() {
  return {
    artifact_key: hash("a"),
    candidate: {
      owner: "standing-parity-fixture",
      expires_at: "2099-07-20T00:00:00.000Z",
      compose_project: "wikijump-candidate-fixture",
      wikijump_commit: git("b"),
      wikijump_tree: git("c"),
      ftml_sha: git("d"),
      profile: "production-build",
      images: { caddy: `sha256:${hash("e")}` },
      config: {
        isolated_overlay_sha256: hash("f"),
        effective_runtime_services_sha256: hash("0"),
      },
      endpoint: {
        local_connect_address: "127.0.0.1",
        port: 18443,
      },
    },
  };
}

function caddyInspect(candidate = identity()) {
  return {
    Id: hash("1"),
    Image: candidate.candidate.images.caddy,
    Config: {
      Labels: {
        "com.docker.compose.project": candidate.candidate.compose_project,
        "com.rokurolize.wikijump.owner": candidate.candidate.owner,
        "com.rokurolize.wikijump.sha": candidate.candidate.wikijump_commit,
        "com.rokurolize.wikijump.tree": candidate.candidate.wikijump_tree,
        "com.rokurolize.wikijump.ftml_sha": candidate.candidate.ftml_sha,
        "com.rokurolize.wikijump.artifact_key": candidate.artifact_key,
        "com.rokurolize.wikijump.config_sha256":
          candidate.candidate.config.isolated_overlay_sha256,
        "com.rokurolize.wikijump.runtime_config_sha256":
          candidate.candidate.config.effective_runtime_services_sha256,
        "com.rokurolize.wikijump.profile": candidate.candidate.profile,
        "com.rokurolize.wikijump.expires_at": candidate.candidate.expires_at,
        "com.rokurolize.wikijump.role": "caddy",
      },
      Image: candidate.candidate.images.caddy,
      Env: [],
      Entrypoint: null,
      Cmd: null,
      WorkingDir: "",
      User: "",
      Hostname: "fixture",
      Healthcheck: null,
      ExposedPorts: {},
    },
    State: { Running: true, Status: "running", Health: { Status: "healthy" } },
    Path: "caddy",
    Args: [],
    HostConfig: {
      Binds: [],
      Mounts: [],
      NetworkMode: "fixture_default",
      PortBindings: { "443/tcp": [{ HostIp: "127.0.0.1", HostPort: "18443" }] },
      RestartPolicy: { Name: "unless-stopped" },
      ReadonlyRootfs: false,
      Tmpfs: {},
      CapAdd: [],
      CapDrop: [],
      Privileged: false,
      SecurityOpt: [],
      ExtraHosts: [],
      Dns: [],
      DnsOptions: [],
      DnsSearch: [],
    },
    NetworkSettings: {
      Ports: { "443/tcp": [{ HostIp: "127.0.0.1", HostPort: "18443" }] },
      Networks: {},
    },
  };
}

function preparedFixture() {
  const candidate = identity();
  const inspect = caddyInspect(candidate);
  const effective = effectiveRuntimeServicesSha256([inspect]);
  candidate.candidate.config.effective_runtime_services_sha256 = effective;
  inspect.Config.Labels["com.rokurolize.wikijump.runtime_config_sha256"] =
    effective;
  return { candidate, inspect };
}

async function observe(
  candidate = identity(),
  inspect = caddyInspect(candidate),
  now = "2026-07-20T00:00:00.000Z",
) {
  return await observeCandidateRuntimeIdentity({
    identity: candidate,
    identitySha256: hash("9"),
    listContainers: async () => ["fixture-caddy"],
    inspectContainer: async () => inspect,
    now: () => now,
  });
}

test("runtime observation binds the actual candidate caddy image, labels, and loopback publication", async () => {
  const { candidate, inspect } = preparedFixture();
  const observation = await observe(candidate, inspect);
  const validated = validateCandidateRuntimeObservation(
    observation,
    candidate,
    {
      identitySha256: hash("9"),
    },
  );
  assert.equal(validated.status, "bound");
  assert.equal(
    validated.services[0].image_id,
    candidate.candidate.images.caddy,
  );
  assert.deepEqual(validated.services[0].https_binding, {
    container_port: "443/tcp",
    host_address: "127.0.0.1",
    host_port: 18443,
  });
});

test("runtime observation fails closed for a mutable endpoint, changed image, missing identity label, or changed effective configuration", async () => {
  const { candidate, inspect } = preparedFixture();
  const publicBinding = structuredClone(inspect);
  publicBinding.NetworkSettings.Ports["443/tcp"][0].HostIp = "0.0.0.0";
  await assert.rejects(
    observe(candidate, publicBinding),
    /does not exactly bind the non-443 loopback endpoint/u,
  );

  const changedImage = structuredClone(inspect);
  changedImage.Image = `sha256:${hash("0")}`;
  await assert.rejects(
    observe(candidate, changedImage),
    /image does not bind/u,
  );

  const missingLabel = structuredClone(inspect);
  delete missingLabel.Config.Labels["com.rokurolize.wikijump.tree"];
  await assert.rejects(
    observe(candidate, missingLabel),
    /runtime label .*tree/u,
  );

  const changedCommand = structuredClone(inspect);
  changedCommand.Config.Cmd = ["unexpected-command"];
  await assert.rejects(
    observe(candidate, changedCommand),
    /effective service configuration does not bind/u,
  );

  const unhealthy = structuredClone(inspect);
  unhealthy.State.Health.Status = "starting";
  await assert.rejects(
    observe(candidate, unhealthy),
    /container is not healthy/u,
  );
});

test("runtime identity must remain unchanged from pre-capture to post-cleanup observation", async () => {
  const { candidate, inspect } = preparedFixture();
  const before = await observe(
    candidate,
    structuredClone(inspect),
    "2026-07-20T00:00:00.000Z",
  );
  const after = await observe(
    candidate,
    structuredClone(inspect),
    "2026-07-20T00:01:00.000Z",
  );
  assert.equal(
    assertStableCandidateRuntimeIdentity(before, after, candidate, {
      identitySha256: hash("9"),
    }).status,
    "bound",
  );

  const changed = structuredClone(inspect);
  changed.Id = hash("2");
  const replaced = await observe(
    candidate,
    changed,
    "2026-07-20T00:01:00.000Z",
  );
  assert.throws(
    () =>
      assertStableCandidateRuntimeIdentity(before, replaced, candidate, {
        identitySha256: hash("9"),
      }),
    /changed during browser parity capture/u,
  );
});
