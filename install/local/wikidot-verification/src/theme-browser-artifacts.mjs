import fs from "node:fs/promises";
import path from "node:path";

export async function prepareThemeArtifactDirectory(directory) {
  const absolute = path.resolve(directory);
  await fs.mkdir(absolute, {recursive: true, mode: 0o700});
  const stat = await fs.lstat(absolute);
  if (!stat.isDirectory() || stat.isSymbolicLink()) {
    throw new Error("theme artifact path must be a real directory");
  }
  if ((stat.mode & 0o077) !== 0) {
    throw new Error(
      "theme artifact directory permissions must deny group and other access",
    );
  }
  return absolute;
}

export async function writePrivateThemeFile(filePath, contents) {
  const handle = await fs.open(filePath, "wx", 0o600);
  try {
    await handle.writeFile(contents);
    await handle.sync();
  } finally {
    await handle.close();
  }
}

async function assertPrivateFile(filePath) {
  const stat = await fs.lstat(filePath);
  if (
    !stat.isFile() ||
    stat.isSymbolicLink() ||
    (stat.mode & 0o077) !== 0
  ) {
    throw new Error("theme artifact file must be a private regular file");
  }
}

export async function writePrivateThemeJson(filePath, value) {
  await writePrivateThemeFile(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

export async function writeThemeViewportArtifacts(directory, result) {
  directory = await prepareThemeArtifactDirectory(directory);
  const artifacts = {
    dom: path.join(directory, "dom.html"),
    screenshot: path.join(directory, "screenshot.png"),
    computed_styles: path.join(directory, "computed-styles.json"),
    web_vitals: path.join(directory, "web-vitals.json"),
    performance_attribution: path.join(
      directory,
      "performance-attribution.json",
    ),
    interactions: path.join(directory, "interactions.json"),
    network_errors: path.join(directory, "network-errors.json"),
    raw_syntax: path.join(directory, "raw-syntax.json"),
    verdict: path.join(directory, "verdict.json"),
  };
  if (result.screenshot_status !== "captured") {
    throw new Error("theme screenshot artifact was not created");
  }
  await assertPrivateFile(artifacts.screenshot);
  await writePrivateThemeFile(artifacts.dom, result.dom ?? "");
  await Promise.all([
    writePrivateThemeJson(artifacts.computed_styles, result.computed_styles),
    writePrivateThemeJson(artifacts.web_vitals, {
      navigation_timing: result.navigation_timing,
      web_vitals: result.web_vitals,
    }),
    writePrivateThemeJson(
      artifacts.performance_attribution,
      result.performance_attribution,
    ),
    writePrivateThemeJson(artifacts.interactions, result.interactions),
    writePrivateThemeJson(artifacts.network_errors, result.errors),
    writePrivateThemeJson(artifacts.raw_syntax, result.raw_syntax),
    writePrivateThemeJson(artifacts.verdict, result.verdict),
  ]);
  return artifacts;
}
