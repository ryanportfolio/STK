import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';

const html = fs.readFileSync('docs/index.html', 'utf8');
const script = fs.readFileSync('docs/app.js', 'utf8');
assert(html.includes('Claude Code + Codex'));
assert(!html.includes('client-legacy'));
assert(!html.includes('id="methodology"'));
assert(html.includes('stk init --auto'));
for (const match of html.matchAll(/href="#([^"]+)"/g)) {
  assert(html.includes(`id="${match[1]}"`), `Broken anchor: ${match[1]}`);
}
async function render(data) {
  const nodes = new Map([...html.matchAll(/id="([^"]+)"[^>]*>([^<]*)/g)]
    .map(([, id, textContent]) => [id, { textContent, classList: { add() {} } }]));
  vm.runInNewContext(script, {
    document: { getElementById: id => nodes.get(id) },
    fetch: async () => ({ ok: data !== null, json: async () => data }),
  });
  await new Promise(resolve => setImmediate(resolve));
  return id => nodes.get(id)?.textContent;
}
const mixed = await render({ stk: {
  clamps: 2, dup_hits: 0, bytes_avoided: 8192, est_tokens: 2048,
  clients: { claude: { bytes_avoided: 2048 }, codex: { bytes_avoided: 4096 }, legacy: { bytes_avoided: 2048 } },
} });
assert.equal(mixed('meter-number'), '2048');
const legacy = await render({ stk: { clamps: 1, bytes_avoided: 1024, est_tokens: 256 } });
const empty = await render({ stk: { clients: {} } });
assert.equal(empty('meter-state'), 'STANDBY');
assert.equal(empty('meter-sub'), 'Snapshot loaded. No clamps or repeat reads yet.');
const failed = await render(null);
assert.equal(failed('meter-state'), 'STANDBY');
assert.equal(failed('meter-sub'), 'Waiting for a published snapshot.');
console.log('PASS: combined totals, zero, and failed-fetch site states; setup and anchors');
