import { devices, type PlaywrightTestConfig } from "@playwright/test"
import browserSupport from "./playwright.browser-support.config"

// Every engine uses the actual HTTPS fixture. Under the default HTTP Vite
// fixture WebKit obeys CSP upgrade-insecure-requests and loses local CSS.
const config: PlaywrightTestConfig = {
  ...browserSupport,
  testMatch: "**/wikidot-typography-https.spec.ts",
  projects: [
    {
      name: "chromium-https",
      use: { ...devices["Desktop Chrome"], ignoreHTTPSErrors: true }
    },
    {
      name: "firefox-https",
      use: { ...devices["Desktop Firefox"], ignoreHTTPSErrors: true }
    },
    {
      name: "webkit-https",
      use: { ...devices["Desktop Safari"], ignoreHTTPSErrors: true }
    },
    {
      name: "mobile-chromium-https",
      use: { ...devices["Pixel 7"], ignoreHTTPSErrors: true }
    }
  ]
}
export default config
