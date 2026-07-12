import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const testDirectory = path.dirname(fileURLToPath(import.meta.url));
const deploymentDirectory = path.resolve(testDirectory, "..");
const documentPath = path.join(deploymentDirectory, "dev.md");
const document = readFileSync(documentPath, "utf8");
const unsafeMaintainerPatterns = [
  /^#\s+adduser\b[^\n]*(?:--disabled-password\b[^\n]*\bmaintainer\b|\bmaintainer\b[^\n]*--disabled-password\b)/im,
  /^#\s+passwd\b[^\n]*(?:(?:--delete|-d)\b[^\n]*\bmaintainer\b|\bmaintainer\b[^\n]*(?:--delete|-d)\b)/im,
  /^\s*PermitEmptyPasswords\s+yes\s*$/im,
  /^\s*KbdInteractiveAuthentication\s+yes\s*$/im,
  /\bNOPASSWD\b/,
];

const containsUnsafeMaintainerGuidance = (value) =>
  unsafeMaintainerPatterns.some((pattern) => pattern.test(value));

test("dev deployment guidance never creates a passwordless sudo maintainer", () => {
  assert.equal(containsUnsafeMaintainerGuidance(document), false);

  assert.match(document, /^# adduser maintainer$/m);
  assert.match(document, /^# gpasswd -a maintainer sudo$/m);
  assert.match(document, /Choose a strong password when prompted/);
  assert.match(
    document,
    /do not leave the sudo-capable `maintainer` account with an empty password/,
  );
  assert.match(document, /^PasswordAuthentication no$/m);
  assert.match(document, /^KbdInteractiveAuthentication no$/m);
  assert.match(document, /^PermitEmptyPasswords no$/m);
  assert.match(document, /^\$ sudo sshd -t$/m);
  assert.match(document, /^\$ sudo systemctl reload ssh\.service$/m);
});

test("maintainer safety guard catches reordered and alternate unsafe guidance", () => {
  for (const unsafe of [
    "# adduser --disabled-password maintainer",
    "# adduser maintainer --disabled-password",
    "# passwd --delete maintainer",
    "# passwd maintainer -d",
    "maintainer ALL=(ALL) NOPASSWD: ALL",
    "KbdInteractiveAuthentication yes",
    "PermitEmptyPasswords yes",
  ]) {
    assert.equal(containsUnsafeMaintainerGuidance(unsafe), true, unsafe);
  }
});

test("relative links in dev deployment guidance resolve to repository documents", () => {
  const links = [...document.matchAll(/\[[^\]]+\]\(([^)]+)\)/g)].map(
    (match) => match[1],
  );
  assert.ok(links.length > 0, "expected at least one documentation link");

  for (const link of links) {
    if (/^(?:https?:|mailto:|#)/.test(link)) continue;
    const target = decodeURIComponent(link.split("#", 1)[0]);
    assert.ok(
      existsSync(path.resolve(deploymentDirectory, target)),
      `broken link: ${link}`,
    );
  }
});
