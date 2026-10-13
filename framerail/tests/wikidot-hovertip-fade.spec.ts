import { expect, test } from "./hermetic-playwright"

const SITE_HEADERS = {
  "X-Wikijump-Site-Id": "6000005",
  "X-Wikijump-Site-Slug": "scp-wiki"
}

// Pinned-animation declarations replayed verbatim from the retained Wikidot
// Basalt asset `basalt-bedrock-min.css` (sha256
// 41cee2998f70df05991e72afce7bdee25f83273a917eda59c4f19231631c9e97,
// captured 2026-10-11, manifest
// scp9506-live-20261012-chromium149). Only the fade declarations are
// replayed: the rule's remaining var()-dependent layout properties belong to
// the full theme and play no part in the fade check.
const BASALT_HOVERTIP_FADE_CSS = [
  ".hovertip{animation:.1325s blt-hover-in forwards}",
  "@keyframes blt-hover-in{from{opacity:0}to{opacity:1}}"
].join("")

// Fixed sample times across the 132.5 ms fade, per the S03 cluster contract.
const PINNED_TIMES_MS = [0, 50, 100, 132.5]

test("Basalt hovertip fade is deterministic when the animation is pinned", async ({
  page
}) => {
  await page.setExtraHTTPHeaders(SITE_HEADERS)
  await page.goto("/wikidot-login-probe")
  await page.addStyleTag({ content: BASALT_HOVERTIP_FADE_CSS })

  const container = page.locator("body > #odialog-hovertips")
  const editTip = container.locator("#edit-button-hovertip")
  // Wait for this tip's attachment (moved into #odialog-hovertips with
  // listeners wired) before hovering: a mouseover that lands before the
  // listeners attach never shows its tip, and a stationary pointer fires
  // no further mouseover once they are attached.
  await expect(container.locator("#edit-button-hovertip")).toHaveCount(1)
  await page.mouse.move(683, 10)
  await page.locator("#edit-button").hover()
  await expect(editTip).toBeVisible()

  // The Basalt theme drives the fade; without it there is no animation.
  const animationIdentity = await editTip.evaluate((tip) => {
    const [animation] = tip.getAnimations()
    return {
      count: tip.getAnimations().length,
      name: animation instanceof CSSAnimation ? animation.animationName : null,
      duration:
        animation instanceof CSSTransition
          ? null
          : (animation?.effect?.getTiming().duration ?? null)
    }
  })
  expect(animationIdentity.count).toBe(1)
  expect(animationIdentity.name).toBe("blt-hover-in")
  expect(animationIdentity.duration).toBe(132.5)
  await expect(editTip).toHaveCSS("animation-name", "blt-hover-in")

  const sweepPinnedFade = () =>
    editTip.evaluate((tip, times) => {
      const [animation] = tip.getAnimations()
      if (!animation) return null
      animation.pause()
      return times.map((time) => {
        animation.currentTime = time
        const box = tip.getBoundingClientRect().toJSON()
        return {
          time,
          opacity: getComputedStyle(tip).opacity,
          box: [box.x, box.y, box.width, box.height]
        }
      })
    }, PINNED_TIMES_MS)

  const firstSweep = await sweepPinnedFade()
  expect(firstSweep).not.toBeNull()
  const secondSweep = await sweepPinnedFade()
  // Pinning the same times twice must read back identical values: the
  // mid-fade phase is then a function of the pin, not of wall-clock time.
  expect(secondSweep).toEqual(firstSweep)

  const samples = firstSweep ?? []
  expect(samples.map((sample) => sample.time)).toEqual(PINNED_TIMES_MS)
  const opacities = samples.map((sample) => Number.parseFloat(sample.opacity))
  expect(opacities[0]).toBe(0)
  expect(opacities[opacities.length - 1]).toBe(1)
  for (const opacity of opacities.slice(1, -1)) {
    expect(opacity).toBeGreaterThan(0)
    expect(opacity).toBeLessThan(1)
  }
  for (let index = 1; index < opacities.length; index += 1) {
    expect(opacities[index]).toBeGreaterThanOrEqual(opacities[index - 1])
  }
  // The fade animates opacity only, so the computed box is identical at
  // every pinned time.
  for (const sample of samples.slice(1)) {
    expect(sample.box).toEqual(samples[0].box)
  }

  // With `forwards` fill the finished fade holds the end state.
  await editTip.evaluate((tip) => {
    tip.getAnimations()[0]?.finish()
  })
  await expect(editTip).toHaveCSS("opacity", "1")
})
