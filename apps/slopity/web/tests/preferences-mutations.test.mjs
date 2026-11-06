import test from 'node:test';
import assert from 'node:assert/strict';
import {
  DEFAULT_UI_PREFERENCES,
  UI_PREFERENCES_KEY,
  loadUiPreferences,
  normalizeUiPreferences,
  saveUiPreferences,
} from '../domain/ui-preferences.js';
import { performBackendMutation } from '../domain/mutation-outcome.js';

test('refresh preferences default to genuine two-second updates', () => {
  assert.deepEqual(normalizeUiPreferences(null), DEFAULT_UI_PREFERENCES);
  assert.deepEqual(normalizeUiPreferences({ autoRefresh: false, refreshIntervalMs: 10000 }),
    { autoRefresh: false, refreshIntervalMs: 10000 });
  assert.deepEqual(normalizeUiPreferences({ autoRefresh: 'false', refreshIntervalMs: 1 }),
    DEFAULT_UI_PREFERENCES);
});

test('settings survive supported local persistence and fail safely without storage', () => {
  const entries = new Map();
  const storage = {
    getItem: (key) => entries.get(key) ?? null,
    setItem: (key, value) => entries.set(key, value),
  };
  const saved = saveUiPreferences(storage, { autoRefresh: false, refreshIntervalMs: 30000 });
  assert.equal(saved.persisted, true);
  assert.deepEqual(JSON.parse(entries.get(UI_PREFERENCES_KEY)), saved.preferences);
  assert.deepEqual(loadUiPreferences(storage), saved.preferences);
  assert.deepEqual(loadUiPreferences({ getItem() { throw new Error('blocked'); } }),
    DEFAULT_UI_PREFERENCES);
  const session = saveUiPreferences({ setItem() { throw new Error('blocked'); } },
    { autoRefresh: false, refreshIntervalMs: 5000 });
  assert.equal(session.persisted, false);
  assert.deepEqual(session.preferences, { autoRefresh: false, refreshIntervalMs: 5000 });
});

test('mutation succeeds only if backend action actually completes', async () => {
  const failure = new Error('port conflict');
  let refreshCalls = 0;
  const result = await performBackendMutation(async () => { throw failure; }, async () => {
    refreshCalls += 1;
    return {};
  });
  assert.equal(result.committed, false);
  assert.equal(result.actionError, failure);
  assert.equal(refreshCalls, 0);
});

test('successful profile mutation stays committed even if dashboard refresh fails', async () => {
  const failure = new Error('dashboard temporarily unavailable');
  let actionCalls = 0;
  const result = await performBackendMutation(async () => { actionCalls += 1; }, async () => {
    throw failure;
  });
  assert.equal(result.committed, true);
  assert.equal(actionCalls, 1);
  assert.equal(result.refreshError, failure);
  assert.equal(result.snapshot, null);
  assert.equal(result.actionError, null);
});

test('successful profile mutation and refresh return the authoritative snapshot', async () => {
  const snapshot = { profiles: [{ id: 'created' }] };
  const result = await performBackendMutation(async () => undefined, async () => snapshot);
  assert.equal(result.committed, true);
  assert.equal(result.refreshError, null);
  assert.equal(result.snapshot, snapshot);
});
