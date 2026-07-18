const DEPLOYMENT_ENVIRONMENTS = new Set(["local", "dev", "prod"])

/**
 * @param {{ framerailEnv?: string | null; nodeEnv?: string | null }} [input]
 * @returns {"local" | "dev" | "prod"}
 */
export const parseDeploymentEnvironment = ({
  framerailEnv = process.env.FRAMERAIL_ENV,
  nodeEnv = process.env.NODE_ENV
} = {}) => {
  if (framerailEnv === undefined || framerailEnv === null || framerailEnv === "") {
    return nodeEnv === "development" ? "local" : "prod"
  }
  if (!DEPLOYMENT_ENVIRONMENTS.has(framerailEnv)) {
    throw new Error(`Invalid FRAMERAIL_ENV: ${JSON.stringify(framerailEnv)}`)
  }
  return /** @type {"local" | "dev" | "prod"} */ (framerailEnv)
}

/**
 * @param {{
 *   csrfCheckOrigin?: string | null;
 *   deploymentEnvironment?: "local" | "dev" | "prod";
 * }} [input]
 * @returns {boolean}
 */
export const parseCsrfCheckOrigin = ({
  csrfCheckOrigin = process.env.FRAMERAIL_CSRF_CHECK_ORIGIN,
  deploymentEnvironment = parseDeploymentEnvironment()
} = {}) => {
  if (
    csrfCheckOrigin === undefined ||
    csrfCheckOrigin === null ||
    csrfCheckOrigin === ""
  ) {
    return deploymentEnvironment !== "local"
  }
  if (csrfCheckOrigin === "true") return true
  if (csrfCheckOrigin === "false") return false
  throw new Error(
    `Invalid FRAMERAIL_CSRF_CHECK_ORIGIN: ${JSON.stringify(csrfCheckOrigin)}`
  )
}
