import test from 'node:test';
import assert from 'node:assert/strict';
import {
  clientValidateProfile,
  cloneIdentity,
  draftToProfile,
  emptyDraft,
  nextAvailablePort,
  nextProfileId,
  profileToDraft,
} from '../domain/profile.js';
import { profileLifecycle } from '../domain/runtime.js';
import { collectResourceWarnings, deviceTelemetryRows, recoveryMessages } from '../domain/resources.js';

const readyRuntime = { runtime: 'built-in-http', available: true, reason: 'Registered.' };
const unavailableJava = { runtime: 'java', available: false, reason: 'No verified runtime adapter is registered.' };

function profile(overrides = {}) {
  return {
    id: 'http-1',
    name: 'HTTP one',
    runtime: 'built-in-http',
    executable: null,
    arguments: [],
    workingDirectory: null,
    port: 8080,
    memoryMib: 128,
    networkScope: 'loopback',
    enabled: true,
    ...overrides,
  };
}

test('create profile draft keeps loopback and BuiltInHttp as safe defaults', () => {
  const draft = emptyDraft(9090);
  draft.id = 'safe-1';
  draft.name = 'Safe server';
  const result = draftToProfile(draft);
  assert.equal(result.runtime, 'built-in-http');
  assert.equal(result.networkScope, 'loopback');
  assert.equal(result.port, 9090);
  assert.equal(result.executable, null);
  assert.deepEqual(result.arguments, []);
});

test('external runtime editing remains structured configuration only', () => {
  const draft = profileToDraft(profile({
    runtime: 'java',
    executable: '/usr/bin/java',
    arguments: ['-jar', 'paper.jar'],
    workingDirectory: '/srv/paper',
    enabled: false,
  }));
  const result = draftToProfile(draft);
  assert.equal(result.executable, '/usr/bin/java');
  assert.deepEqual(result.arguments, ['-jar', 'paper.jar']);
  assert.equal(result.enabled, false);
});

test('client validation renders actionable field ownership and LAN warning', () => {
  const issues = clientValidateProfile(profile({ id: '../bad', port: 70000, networkScope: 'lan' }), {
    profiles: [],
    runtimes: [readyRuntime],
  });
  assert.ok(issues.some((issue) => issue.field === 'id' && issue.severity === 'error'));
  assert.ok(issues.some((issue) => issue.field === 'port' && issue.severity === 'error'));
  assert.ok(issues.some((issue) => issue.field === 'networkScope' && issue.severity === 'warning'));
});

test('unsupported runtime validation is a warning, not fake run support', () => {
  const issues = clientValidateProfile(profile({ runtime: 'java', enabled: false }), {
    profiles: [],
    runtimes: [readyRuntime, unavailableJava],
  });
  assert.match(issues.find((issue) => issue.field === 'runtime')?.message ?? '', /configuration-only/i);
});

test('enabled BuiltInHttp profile is runnable when backend reports adapter available', () => {
  const lifecycle = profileLifecycle(profile(), { runtimes: [readyRuntime], servers: [] });
  assert.equal(lifecycle.runnable, true);
  assert.equal(lifecycle.startDisabledReason, null);
});

test('disabled BuiltInHttp profile cannot dispatch start', () => {
  const lifecycle = profileLifecycle(profile({ enabled: false }), { runtimes: [readyRuntime], servers: [] });
  assert.equal(lifecycle.runnable, false);
  assert.match(lifecycle.startDisabledReason, /enable/i);
});

test('unsupported runtime cannot dispatch start even when profile is enabled', () => {
  const lifecycle = profileLifecycle(profile({ runtime: 'java' }), { runtimes: [readyRuntime, unavailableJava], servers: [] });
  assert.equal(lifecycle.runnable, false);
  assert.match(lifecycle.startDisabledReason, /No verified runtime adapter/i);
});

test('active runtime state exposes stop and desired state', () => {
  const lifecycle = profileLifecycle(profile(), {
    runtimes: [readyRuntime],
    servers: [{ serverId: 'http-1', state: 'running', desiredState: 'running' }],
  });
  assert.equal(lifecycle.active, true);
  assert.equal(lifecycle.canStop, true);
  assert.equal(lifecycle.state, 'running');
  assert.equal(lifecycle.server.desiredState, 'running');
});

test('resource warnings are deduplicated across plan and accounting', () => {
  const warning = { code: 'low-cpu-headroom', message: 'Low CPU headroom.' };
  const warnings = collectResourceWarnings({ resourcePlan: { warnings: [warning] }, resourceAccounting: { warnings: [warning] } });
  assert.deepEqual(warnings, [warning]);
});

test('unavailable telemetry is displayed as unavailable rather than zero', () => {
  const rows = Object.fromEntries(deviceTelemetryRows({ capability: {}, deviceTelemetry: {} }));
  assert.equal(rows['Available RAM'], 'Unavailable');
  assert.equal(rows.Battery, 'Unavailable');
  assert.equal(rows.Charging, 'Unavailable');
  assert.equal(rows['Battery temperature'], 'Unavailable');
});

test('profile recovery notices are surfaced from backend snapshot', () => {
  assert.deepEqual(recoveryMessages({ profileRecoveryNotices: [
    { code: 'backup-recovered', message: 'Recovered profiles from the last good backup.' },
  ] }), ['Recovered profiles from the last good backup.']);
});

test('next profile IDs, clone IDs, and ports avoid existing reservations', () => {
  const profiles = [profile(), profile({ id: 'http-2', port: 8081 }), profile({ id: 'http-1-copy', port: 8082 })];
  assert.equal(nextProfileId(profiles, 'http'), 'http-3');
  assert.equal(nextAvailablePort(profiles, 8080), 8083);
  assert.deepEqual(cloneIdentity(profiles[0], profiles), { id: 'http-1-copy-2', name: 'HTTP one copy' });
});
