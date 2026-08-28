import test from 'node:test';
import assert from 'node:assert/strict';
import { issueList } from '../components/dom.js';
import { renderServerCard } from '../components/server-card.js';
import { FakeDocument, findByClass, findByTag } from './fake-dom.mjs';

const document = new FakeDocument();
const ready = { runtime: 'built-in-http', available: true, reason: 'Registered.' };
const unavailable = { runtime: 'java', available: false, reason: 'No verified Java adapter is registered.' };

function profile(overrides = {}) {
  return {
    id: 'http-1',
    name: 'HTTP one',
    runtime: 'built-in-http',
    port: 8080,
    memoryMib: 128,
    networkScope: 'loopback',
    enabled: true,
    ...overrides,
  };
}

test('profile rendering communicates runtime, port, memory, and lifecycle state', () => {
  const card = renderServerCard(document, profile(), { runtimes: [ready], servers: [] });
  assert.match(card.textContent, /HTTP one/);
  assert.match(card.textContent, /Built-in HTTP/);
  assert.match(card.textContent, /:8080/);
  assert.match(card.textContent, /128 MiB/);
  assert.match(card.textContent, /Stopped/);
});

test('unavailable runtime rendering disables lifecycle button and explains why', () => {
  const card = renderServerCard(document, profile({ runtime: 'java' }), { runtimes: [ready, unavailable], servers: [] });
  const primary = findByClass(card, 'primary')[0];
  assert.equal(primary.textContent, 'Unavailable');
  assert.equal(primary.disabled, true);
  assert.match(card.textContent, /No verified Java adapter is registered/);
});

test('enabled versus disabled profiles render correct start controls', () => {
  const enabled = renderServerCard(document, profile(), { runtimes: [ready], servers: [] });
  const disabled = renderServerCard(document, profile({ enabled: false }), { runtimes: [ready], servers: [] });
  assert.equal(findByClass(enabled, 'primary')[0].disabled, false);
  assert.equal(findByClass(disabled, 'primary')[0].disabled, true);
  assert.match(findByClass(disabled, 'primary')[0].getAttribute('title'), /Enable this profile/i);
});

test('active server renders stop control and dispatches stop callback', () => {
  let stopped = null;
  const current = profile();
  const card = renderServerCard(document, current, {
    runtimes: [ready],
    servers: [{ serverId: 'http-1', state: 'running', desiredState: 'running' }],
  }, { onStop: (value) => { stopped = value.id; } });
  const danger = findByClass(card, 'danger')[0];
  assert.equal(danger.textContent, 'Stop');
  danger.dispatch('click');
  assert.equal(stopped, 'http-1');
});

test('validation and command errors render as text nodes', () => {
  const list = issueList(document, [{ severity: 'error', message: 'Port bind failed: <b>address in use</b>' }]);
  assert.match(list.textContent, /<b>address in use<\/b>/);
  assert.equal(findByTag(list, 'b').length, 0);
});

test('resource warning rendering preserves backend message as text', () => {
  const list = issueList(document, [{ severity: 'warning', message: 'Reserved server memory exceeds safe budget.' }]);
  assert.match(list.textContent, /Reserved server memory exceeds safe budget/);
  assert.equal(findByClass(list, 'issue-warning').length, 1);
});

test('dangerous user profile values cannot become executable HTML', () => {
  const attack = '<img src=x onerror=alert(1)><script>globalThis.pwned=1</script>';
  const card = renderServerCard(document, profile({ name: attack }), { runtimes: [ready], servers: [] });
  assert.match(card.textContent, /<img src=x/);
  assert.equal(findByTag(card, 'img').length, 0);
  assert.equal(findByTag(card, 'script').length, 0);
});

test('dialog helpers restore focus to the invoking control', async () => {
  const { showDialog, closeDialog } = await import('../views/dialogs.js');
  let restored = false;
  let focusedInside = false;
  const origin = { isConnected: true, focus() { restored = true; } };
  const target = { focus() { focusedInside = true; } };
  const dialog = {
    open: false,
    ownerDocument: { activeElement: origin },
    showModal() { this.open = true; },
    close() { this.open = false; },
    querySelector() { return target; },
  };
  showDialog(dialog, target);
  await new Promise((resolve) => setTimeout(resolve, 1));
  assert.equal(focusedInside, true);
  closeDialog(dialog);
  await new Promise((resolve) => setTimeout(resolve, 1));
  assert.equal(restored, true);
});
