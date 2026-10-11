import { devices, type PlaywrightTestConfig } from "@playwright/test"

import baseConfig from "./playwright.config"

const config: PlaywrightTestConfig = {
  ...baseConfig,
  testMatch: "**/reserved-admin-route-aliases.spec.ts",
  projects: [
    { name: "chromium", use: { ...devices["Desktop Chrome"] } },
    { name: "firefox", use: { ...devices["Desktop Firefox"] } },
    { name: "webkit", use: { ...devices["Desktop Safari"] } }
  ]
}

export default config
