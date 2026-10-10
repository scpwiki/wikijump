import { devices, type PlaywrightTestConfig } from "@playwright/test"

import baseConfig from "./playwright.config"

const withBaseUse = (device: (typeof devices)[keyof typeof devices]) => ({
  ...(baseConfig.use ?? {}),
  ...device
})

const config: PlaywrightTestConfig = {
  ...baseConfig,
  testMatch:
    "**/{append-file-upload-accessible-names,append-file-upload-accessible-names-ja,auth-accessible-names,browser-support,edit-meta-newtag-mobile,forum-start-routes,module-accessible-names,printer-friendly-print-toolbar}.spec.ts",
  webServer: [
    ...(baseConfig.webServer
      ? Array.isArray(baseConfig.webServer)
        ? baseConfig.webServer
        : [baseConfig.webServer]
      : []),
    {
      command: "sh tests/start-playwright-https-vite.sh",
      port: Number(process.env.PLAYWRIGHT_HTTPS_APP_PORT ?? "4373"),
      reuseExistingServer: false,
      timeout: 120_000,
      env: {
        FRAMERAIL_VITE_TEST_CACHE: "1",
        PLAYWRIGHT_HTTPS_APP_PORT: process.env.PLAYWRIGHT_HTTPS_APP_PORT ?? "4373",
        PLAYWRIGHT_FIXTURE_PORT: process.env.PLAYWRIGHT_FIXTURE_PORT ?? "42747",
        DEEPWELL_RPC_TOKEN: process.env.DEEPWELL_RPC_TOKEN ?? "0".repeat(64)
      }
    }
  ],
  projects: [
    {
      name: "chromium",
      use: withBaseUse(devices["Desktop Chrome"])
    },
    {
      name: "firefox",
      use: withBaseUse(devices["Desktop Firefox"])
    },
    {
      name: "webkit",
      use: withBaseUse(devices["Desktop Safari"])
    },
    {
      name: "webkit-https-edit-meta",
      testMatch:
        "**/{append-file-upload-accessible-names,append-file-upload-accessible-names-ja,edit-meta-newtag-mobile}.spec.ts",
      use: {
        ...withBaseUse(devices["Desktop Safari"]),
        ignoreHTTPSErrors: true
      }
    },
    {
      name: "mobile-chromium",
      use: withBaseUse(devices["Pixel 7"])
    }
  ]
}

export default config
