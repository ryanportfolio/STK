import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';

const html = fs.readFileSync('docs/index.html', 'utf8');
const script = fs.readFileSync('docs/app.js', 'utf8');
assert(html.includes('Claude Code and Codex'));
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
assert.equal(mixed('client-codex'), '4.0 KB');
assert.equal(mixed('client-claude'), '2.0 KB');
assert.equal(mixed('client-legacy'), '2.0 KB');
assert.equal(mixed('meter-number'), '2048');
const legacy = await render({ stk: { clamps: 1, bytes_avoided: 1024, est_tokens: 256 } });
assert.equal(legacy('client-codex'), 'Unattributed');
assert.equal(legacy('client-legacy'), '1.0 KB');
const empty = await render({ stk: { clients: {} } });
assert.equal(empty('client-codex'), '0 B');
assert.equal(empty('meter-state'), 'STANDBY');
const failed = await render(null);
assert.equal(failed('meter-state'), 'STANDBY');
console.log('PASS: mixed-client, legacy, zero, and failed-fetch site states; setup and anchors');
