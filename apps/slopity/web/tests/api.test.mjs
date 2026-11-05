import test from 'node:test';
import assert from 'node:assert/strict';
import { createSlopityApi } from '../api/tauri.js';

function recorder(result = undefined) {
  const calls = [];
  const invoke = async (command, args) => {
    calls.push({ command, args });
    return result;
  };
  return { api: createSlopityApi(invoke), calls };
}

const profile = { id: 'http-1', name: 'HTTP', runtime: 'built-in-http' };

test('create profile flow dispatches typed profile to backend command', async () => {
  const { api, calls } = recorder([]);
  await api.createProfile(profile);
  assert.deepEqual(calls, [{ command: 'create_server_profile', args: { profile } }]);
});

test('edit profile flow dispatches update command', async () => {
  const { api, calls } = recorder([]);
  await api.updateProfile(profile);
  assert.deepEqual(calls, [{ command: 'update_server_profile', args: { profile } }]);
});

test('delete flow dispatches only profile ID', async () => {
  const { api, calls } = recorder([]);
  await api.deleteProfile('http-1');
  assert.deepEqual(calls, [{ command: 'delete_server_profile', args: { id: 'http-1' } }]);
});

test('start command dispatch is restricted to built-in HTTP command surface', async () => {
  const { api, calls } = recorder({ serverId: 'http-1', state: 'running' });
  await api.startServer('http-1');
  assert.deepEqual(calls, [{ command: 'start_builtin_http_server', args: { id: 'http-1' } }]);
});

test('stop command dispatch uses built-in HTTP stop command', async () => {
  const { api, calls } = recorder({ serverId: 'http-1', state: 'stopped' });
  await api.stopServer('http-1');
  assert.deepEqual(calls, [{ command: 'stop_builtin_http_server', args: { id: 'http-1' } }]);
});

test('backend command failures propagate to error UX instead of being swallowed', async () => {
  const api = createSlopityApi(async () => { throw new Error('port bind failed: address already in use'); });
  await assert.rejects(() => api.startServer('http-1'), /port bind failed/);
});

test('missing Tauri bridge produces an explicit command failure', () => {
  const api = createSlopityApi(null);
  assert.throws(() => api.dashboard(), /native Tauri bridge is unavailable/i);
});
