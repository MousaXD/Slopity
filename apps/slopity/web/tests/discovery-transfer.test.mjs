import test from 'node:test';
import assert from 'node:assert/strict';
import { selectProfiles, profileStatus } from '../domain/profile-query.js';
import { serializeProfiles, parseProfileImport, MAX_IMPORT_BYTES } from '../domain/profile-transfer.js';
import { renderImportPreview } from '../views/import-preview.js';
import { FakeDocument, findByTag } from './fake-dom.mjs';

const http = { id: 'http-1', name: 'HTTP 1', runtime: 'built-in-http', networkScope: 'loopback',
  port: 8080, memoryMib: 128, enabled: true, executable: null, workingDirectory: null, arguments: [] };
const java = { ...http, id: 'java-1', name: 'Java config', runtime: 'java', port: 8081, enabled: false };
const active = { ...http, id: 'http-2', name: 'HTTP 2', port: 8082 };
const snapshot = {
  profiles: [java, active, http],
  runtimes: [{ runtime: 'built-in-http', available: true, reason: 'Ready' },
    { runtime: 'java', available: false, reason: 'Not installed' }],
  servers: [{ serverId: 'http-2', state: 'running' }],
};

test('search matches name/id/runtime/port without mutating backend profile order', () => {
  assert.deepEqual(selectProfiles(snapshot, { query: '8081' }).map((p) => p.id), ['java-1']);
  assert.deepEqual(selectProfiles(snapshot, { query: 'HTTP' }).map((p) => p.id), ['http-1', 'http-2']);
  assert.deepEqual(snapshot.profiles.map((p) => p.id), ['java-1', 'http-2', 'http-1']);
});

test('status and runtime filtering use real runtime registration and observed state', () => {
  assert.equal(profileStatus(active, snapshot), 'active');
  assert.equal(profileStatus(java, snapshot), 'disabled');
  assert.deepEqual(selectProfiles(snapshot, { status: 'active' }).map((p) => p.id), ['http-2']);
  assert.deepEqual(selectProfiles(snapshot, { status: 'disabled', runtime: 'java' }).map((p) => p.id), ['java-1']);
  assert.deepEqual(selectProfiles(snapshot, { status: 'unavailable' }), []);
  assert.deepEqual(selectProfiles({ ...snapshot, profiles: [{ ...java, enabled: true }] },
    { status: 'unavailable' }).map((p) => p.id), ['java-1']);
});

test('sort by port and observed status gives stable deterministic results', () => {
  assert.deepEqual(selectProfiles(snapshot, { sort: 'port' }).map((p) => p.port), [8080, 8081, 8082]);
  assert.deepEqual(selectProfiles(snapshot, { sort: 'state' }).map((p) => p.id), ['http-2', 'java-1', 'http-1']);
});

test('versioned export round-trips profile metadata as JSON data only', () => {
  const serialized = serializeProfiles([http, java]);
  const parsed = parseProfileImport(serialized, []);
  assert.equal(parsed.importable, 2);
  assert.deepEqual(parsed.entries.map((entry) => entry.profile.enabled), [false, false]);
  assert.match(serialized, /"version": 1/);
});

test('import previews collisions and duplicate file IDs without replacing existing profiles', () => {
  const text = serializeProfiles([http, java, { ...java, name: 'Duplicate Java' }]);
  const parsed = parseProfileImport(text, [http]);
  assert.equal(parsed.importable, 1);
  assert.equal(parsed.skipped, 2);
  assert.match(parsed.entries[0].reason, /already exists/);
  assert.match(parsed.entries[2].reason, /Duplicate ID/);
});

test('import rejects untrusted fields, invalid schema, oversized input and executable attachments to HTTP', () => {
  const base = JSON.parse(serializeProfiles([http]));
  assert.throws(() => parseProfileImport(JSON.stringify({ ...base, version: 999 })), /Unsupported profile file/);
  assert.throws(() => parseProfileImport(JSON.stringify({ ...base, profiles: [{ ...http, code: 'run()' }] })), /unsupported fields/);
  assert.throws(() => parseProfileImport(JSON.stringify({ ...base, profiles: [{ ...http, executable: '/bin/sh' }] })), /cannot attach external executables/);
  assert.throws(() => parseProfileImport('x'.repeat(MAX_IMPORT_BYTES + 1)), /256 KiB/);
  assert.throws(() => parseProfileImport('{'), /not valid JSON/);
});

test('preview renders untrusted profile text safely without HTML injection', () => {
  const document = new FakeDocument();
  const target = document.createElement('div');
  renderImportPreview(document, target, { importable: 1, skipped: 0, entries: [
    { profile: { ...http, name: '<img src=x onerror=alert(1)>' }, reason: null },
  ] });
  assert.equal(findByTag(target, 'img').length, 0);
  assert.match(target.textContent, /<img src=x/);
  assert.match(target.textContent, /disabled/);
});
