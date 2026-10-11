// Explicit local-only, read-only acquisition. Never used in normal CI.
// Usage: node capture-local.cjs /absolute/output/directory
const { chromium, firefox, webkit, devices } = require('../../../../framerail/node_modules/@playwright/test')
const { mkdirSync, writeFileSync } = require('node:fs')
const path = require('node:path')
const output = process.argv[2]
if (!output || !path.isAbsolute(output)) {
  console.error('Usage: node capture-local.cjs /absolute/output/directory')
  process.exit(2)
}
const url = 'https://scp-wiki.wikijump.localhost/theme:yossistyle'
mkdirSync(output, { recursive: true })
const results = []
;(async () => {
  for (const [engine, type, device] of [
    ['chromium', chromium, null],
    ['firefox', firefox, null],
    ['webkit', webkit, null],
    ['mobile-chromium', chromium, devices['Pixel 7']]
  ]) {
    const browser = await type.launch({ headless: true })
    const context = await browser.newContext({
      ignoreHTTPSErrors: true,
      ...(device || { viewport: { width: 1280, height: 800 } })
    })
    const page = await context.newPage()
    const failures = []
    page.on('requestfailed', request => failures.push({
      url: request.url(),
      reason: request.failure()?.errorText || ''
    }))
    try {
      const response = await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 20_000 })
      if (engine === 'chromium' && response) {
        writeFileSync(path.join(output, 'local-rendered.html'), await response.body())
      }
      await page.waitForTimeout(2500)
      const data = await page.evaluate(() => {
        const sample = selector => {
          const element = document.querySelector(selector)
          if (!element) return null
          const style = getComputedStyle(element)
          const rect = element.getBoundingClientRect()
          return {
            text: element.textContent.trim().slice(0, 100),
            css: Object.fromEntries(['font-family', 'font-size', 'font-weight', 'line-height', 'text-rendering'].map(key => [key, style.getPropertyValue(key)])),
            rect: { x: rect.x, y: rect.y, width: rect.width, height: rect.height }
          }
        }
        return {
          title: document.title,
          body: sample('body'),
          content: sample('#page-content p'),
          titleElement: sample('#page-title'),
          stylesheets: document.querySelectorAll('link[rel~=stylesheet]').length,
          articleText: document.querySelector('#page-content')?.innerText.slice(0, 1400) || '',
          sourceStyles: [...document.querySelectorAll('link[rel~=stylesheet]')].map(link => ({
            href: link.href, loaded: !!link.sheet
          }))
        }
      })
      results.push({
        engine, url, status: response?.status(), viewport: page.viewportSize(),
        ...data, requestFailures: failures
      })
      await page.screenshot({ path: path.join(output, `local-${engine}.png`), fullPage: true })
      console.log(`${engine}: ${response?.status()} ${data.title} (${failures.length} failed requests)`)
    } catch (error) {
      results.push({ engine, error: String(error), requestFailures: failures })
      console.error(`${engine}: ${error}`)
      process.exitCode = 1
    } finally {
      await browser.close()
    }
  }
  writeFileSync(path.join(output, 'local-wikijump-observation.json'), JSON.stringify(results, null, 2) + '\n')
})().catch(error => { console.error(error); process.exitCode = 1 })
