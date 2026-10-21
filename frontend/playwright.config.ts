import { defineConfig, devices } from '@playwright/test'
import { defineBddConfig } from 'playwright-bdd'

const PORT = 5182

const testDir = defineBddConfig({
  features: '../features/ui_*.feature',
  steps: ['e2e/support/fixtures.ts', 'e2e/steps/*.ts'],
  featuresRoot: '../features',
})

export default defineConfig({
  testDir,
  fullyParallel: true,
  workers: 4,
  timeout: 60_000,
  expect: { timeout: 5_000 },
  reporter: [['list']],
  use: {
    baseURL: `http://127.0.0.1:${PORT}`,
    ...devices['Desktop Chrome'],
    viewport: { width: 1280, height: 800 },
    // 預設以 Windows 平台跑，「Ctrl」即 Control；macOS 情境另外覆寫
    userAgent:
      'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36',
    trace: 'retain-on-failure',
  },
  webServer: {
    command: `npx vite --port ${PORT} --strictPort --host 127.0.0.1`,
    url: `http://127.0.0.1:${PORT}`,
    reuseExistingServer: true,
    timeout: 60_000,
  },
})
