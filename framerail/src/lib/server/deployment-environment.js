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
