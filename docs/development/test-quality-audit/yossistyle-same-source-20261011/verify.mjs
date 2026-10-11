// Offline receipt verifier only: NEVER reaches Wikidot, Wikijump or a CDN.
// A successful exit proves the retained narrow observations are consistent,
// NOT that whole-page Wikidot parity or source/version identity is proved.
import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { gunzipSync } from 'node:zlib'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import path from 'node:path'

const directory = path.dirname(fileURLToPath(import.meta.url))
const bytes = (name) => readFileSync(path.join(directory, name))
const read = (name) => JSON.parse(bytes(name).toString('utf8'))
const sha = (data) => createHash('sha256').update(data).digest('hex')
const expectedSource = '561228038e46e76ab45e61d95570e82519bceea1aa5233addbbec324601ca4d4'
const expectedLive = '632948f633aea578b55225958521c1f2607e3a9ff873c7e17be2f4db811be08e'
const source = gunzipSync(bytes('source.wikidot.txt.gz'))
assert.equal(sha(source), expectedSource, 'pinned corpus source changed')
assert.equal(sha(bytes('live-wikidot-observation.json')), expectedLive, 'retained Wikidot observation changed')
const html = gunzipSync(bytes('local-rendered.html.gz')).toString('utf8')
const embedded = html.match(/wikitext:("(?:\\.|[^"\\])*")/)
assert.ok(embedded, 'local SSR page has no embedded authored source')
assert.equal(sha(Buffer.from(JSON.parse(embedded[1]), 'utf8')), expectedSource, 'locally imported source differs from corpus')
const reference = read('live-wikidot-observation.json')
const candidate = read('local-wikijump-observation.json')
const engines = ['chromium', 'firefox', 'webkit', 'mobile-chromium']
assert.deepEqual(reference.map(row => row.engine), engines)
assert.deepEqual(candidate.map(row => row.engine), engines)
const rows = []
for (let i = 0; i < engines.length; i++) {
  const live = reference[i]
  const local = candidate[i]
  assert.equal(live.status, 200)
  assert.equal(local.status, 200)
  assert.deepEqual(local.viewport, live.viewport, `${live.engine} viewport differs`)
  assert.equal(live.url, 'https://scp-wiki.wikidot.com/theme:yossistyle')
  assert.equal(local.url, 'https://scp-wiki.wikijump.localhost/theme:yossistyle')
  assert.equal(live.title, 'YOSSISTYLE CSS THEME - SCP Foundation')
  assert.equal(local.title, 'YOSSISTYLE CSS THEME | SCP Foundation')
  for (const key of ['content', 'titleElement']) {
    assert.equal(local[key].text, live[key].text, `${live.engine} ${key} text`)
    assert.deepEqual(local[key].css, live[key].css, `${live.engine} ${key} computed CSS`)
    assert.deepEqual(local[key].rect, live[key].rect, `${live.engine} ${key} rectangle`)
  }
  assert.deepEqual(local.body.css, live.body.css, `${live.engine} body computed CSS`)
  rows.push({
    engine: live.engine,
    title_and_first_paragraph: 'EXACT',
    body_height_delta_px: local.body.rect.height - live.body.rect.height,
    stylesheet_count_live: live.stylesheets,
    stylesheet_count_local: local.stylesheets,
    local_resource_failures: local.requestFailures.length
  })
}
console.log(JSON.stringify({ source_sha256: expectedSource, scope: 'first paragraph / title CSSOM and geometry only', full_page_parity: 'UNVERIFIED', source_freshness: 'UNVERIFIED', rows }, null, 2))
