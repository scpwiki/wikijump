import { parseDeploymentEnvironment } from "./deployment-environment.js"

/**
 * Authoritative platform legal destinations.
 *
 * The account-creation disclosure on `/-/register` has to name the Terms
 * of Service and Privacy Policy that govern creating an account, so these
 * are platform/instance destinations, never the wiki the reader happens to
 * be viewing: a legal link that resolves to the current site's root is not
 * a legal link at all (issue #2068).
 *
 * A deployment is the source of truth for its own policies. Operators
 * point an instance at its own policies with
 * `WIKIJUMP_TERMS_OF_SERVICE_URL` and `WIKIJUMP_PRIVACY_POLICY_URL`; when
 * an operator does not configure them, the documented platform
 * destinations for the deployment environment are used, which mirrors how
 * the session cookie already derives the platform host. A self-hosted
 * instance that serves its policies from another host must set the
 * environment variables instead of inheriting the public ones.
 *
 * Destinations are validated as absolute `https:` URLs. Anything else is a
 * configuration defect and fails closed, because a relative or
 * scheme-relative destination would silently resolve against the viewed
 * wiki's origin.
 */

const PLATFORM_LEGAL_HOSTS = {
  local: "wikijump.localhost",
  dev: "wikijump.dev",
  prod: "wikijump.com"
}

const PLATFORM_LEGAL_PATHS = {
  termsOfService: "/legal/terms-of-service",
  privacyPolicy: "/legal/privacy-policy"
}

const LEGAL_DESTINATION_ENVIRONMENT_VARIABLES = {
  termsOfService: "WIKIJUMP_TERMS_OF_SERVICE_URL",
  privacyPolicy: "WIKIJUMP_PRIVACY_POLICY_URL"
}

export type LegalDestinationKey = keyof typeof PLATFORM_LEGAL_PATHS
export type LegalDestinations = Record<LegalDestinationKey, string>

/**
 * @param {string | undefined} value
 * @param {string} environmentVariable
 * @returns {string | null}
 */
const configuredDestination = (
  value: string | undefined,
  environmentVariable: string
): string | null => {
  if (value === undefined || value === null || value.trim() === "") return null
  let url: URL
  try {
    url = new URL(value.trim())
  } catch {
    throw new Error(`Invalid ${environmentVariable}: not an absolute URL`)
  }
  // A relative or scheme-relative destination would resolve against the
  // viewed wiki's origin, which is exactly the placeholder this replaces.
  if (url.protocol !== "https:") {
    throw new Error(`Invalid ${environmentVariable}: must be an https URL`)
  }
  return url.toString()
}

/**
 * Resolves the instance's authoritative legal destinations.
 *
 * @param {{
 *   deploymentEnvironment?: "local" | "dev" | "prod"
 *   env?: Record<string, string | undefined>
 * }} [input]
 * @returns {LegalDestinations}
 * @throws {Error} If a configured destination is not an absolute https
 *   URL.
 */
export const legalDestinations = ({
  deploymentEnvironment = parseDeploymentEnvironment(),
  env = process.env
}: {
  deploymentEnvironment?: "local" | "dev" | "prod"
  env?: Record<string, string | undefined>
} = {}): LegalDestinations => {
  const host = PLATFORM_LEGAL_HOSTS[deploymentEnvironment]
  const keys = Object.keys(PLATFORM_LEGAL_PATHS) as LegalDestinationKey[]

  return keys.reduce<LegalDestinations>((destinations, key) => {
    const environmentVariable = LEGAL_DESTINATION_ENVIRONMENT_VARIABLES[key]
    const configured = configuredDestination(
      env[environmentVariable],
      environmentVariable
    )
    destinations[key] = configured ?? `https://${host}${PLATFORM_LEGAL_PATHS[key]}`
    return destinations
  }, {} as LegalDestinations)
}
