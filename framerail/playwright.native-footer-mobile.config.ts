import { devices, type PlaywrightTestConfig } from "@playwright/test"

import baseConfig from "./playwright.config"

const config: PlaywrightTestConfig = {
  ...baseConfig,
  testMatch: "**/native-footer-mobile-engines.spec.ts",
  projects: [
    { name: "chromium", use: { ...baseConfig.use, ...devices["Desktop Chrome"] } },
    { name: "firefox", use: { ...baseConfig.use, ...devices["Desktop Firefox"] } },
    { name: "webkit", use: { ...baseConfig.use, ...devices["Desktop Safari"] } }
  ]
}

export default config
