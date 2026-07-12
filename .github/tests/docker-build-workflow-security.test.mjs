import assert from "node:assert/strict"
import { readFileSync } from "node:fs"
import path from "node:path"
import test from "node:test"
import { fileURLToPath } from "node:url"

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..")
const workflow = (name) =>
  readFileSync(path.join(root, ".github/workflows", name), "utf8")
const buildCallers = ["caddy", "deepwell", "framerail", "wws"].flatMap((service) =>
  ["dev", "prod"].map((tier) => `docker-build-${service}.${tier}.yaml`)
)

test("pull-request Docker callers build without repository credentials", () => {
  for (const name of buildCallers) {
    const source = workflow(name)

    assert.match(source, /^\s*pull_request:$/m, `${name} must retain its PR trigger`)
    assert.match(source, /^\s*workflow_dispatch:$/m, `${name} must retain manual builds`)
    assert.match(source, /docker-build-template\.yaml/, `${name} must use the template`)
    assert.match(
      source,
      /- '\.github\/workflows\/docker-build-template\.yaml'/,
      `${name} must rebuild when the reusable workflow changes`
    )
    assert.doesNotMatch(source, /secrets\.(?:AWS_ECR_ACCESS_KEY|AWS_ECR_SECRET_KEY)/)
    assert.doesNotMatch(source, /^\s+secrets:\s*$/m)
    assert.doesNotMatch(source, /^\s+push:\s*true\s*$/m)
    assert.doesNotMatch(source, /unused-for-build-only-workflow/)
  }
})

test("reusable Docker credentials are optional and consumed only by push", () => {
  const source = workflow("docker-build-template.yaml")
  const workflowSecrets = source.slice(source.indexOf("    secrets:"), source.indexOf("\n\npermissions:"))
  const pushStep = source.slice(source.indexOf("      - name: Push image to ECR"))

  assert.equal((workflowSecrets.match(/required: false/g) ?? []).length, 2)
  assert.doesNotMatch(workflowSecrets, /required: true/)
  assert.match(pushStep, /if: \$\{\{ inputs\.push \}\}/)
  assert.match(pushStep, /secrets\.aws-access-key/)
  assert.match(pushStep, /secrets\.aws-secret-key/)
  assert.doesNotMatch(source.slice(0, source.indexOf("      - name: Push image to ECR")), /secrets\.aws-/)
})

test("deployment callers still opt into push with real credentials", () => {
  for (const tier of ["dev", "prod"]) {
    const source = workflow(`komodo-deploy.${tier}.yaml`)
    assert.match(source, /^\s+push:\s*true\s*$/m)
    assert.match(source, /secrets\.AWS_ECR_ACCESS_KEY/)
    assert.match(source, /secrets\.AWS_ECR_SECRET_KEY/)
  }
})
