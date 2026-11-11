import { availabilityFor, isActiveState, runtimeLabel } from './runtime.js';

const PROFILE_ID = /^[A-Za-z0-9._-]{1,128}$/;

export function emptyDraft(defaultPort = 8080) {
  return {
    id: '',
    name: '',
    runtime: 'built-in-http',
    networkScope: 'loopback',
    port: String(defaultPort),
    memoryMib: '128',
    executable: '',
    workingDirectory: '',
    arguments: '',
    enabled: true,
  };
}

export function profileToDraft(profile) {
  return {
    id: profile.id,
    name: profile.name,
    runtime: profile.runtime,
    networkScope: profile.networkScope,
    port: String(profile.port),
    memoryMib: String(profile.memoryMib),
    executable: profile.executable ?? '',
    workingDirectory: profile.workingDirectory ?? '',
    arguments: (profile.arguments ?? []).join('\n'),
    enabled: Boolean(profile.enabled),
  };
}

export function draftToProfile(draft) {
  const builtIn = draft.runtime === 'built-in-http';
  return {
    id: String(draft.id ?? '').trim(),
    name: String(draft.name ?? '').trim(),
    runtime: draft.runtime,
    executable: builtIn ? null : nullable(draft.executable),
    arguments: builtIn
      ? []
      : String(draft.arguments ?? '')
          .split('\n')
          .map((argument) => argument.trim())
          .filter(Boolean),
    workingDirectory: builtIn ? null : nullable(draft.workingDirectory),
    port: Number(draft.port),
    memoryMib: Number(draft.memoryMib),
    networkScope: draft.networkScope,
    enabled: Boolean(draft.enabled),
  };
}

function nullable(value) {
  const trimmed = String(value ?? '').trim();
  return trimmed ? trimmed : null;
}

export function clientValidateProfile(profile, snapshot = {}, originalId = null) {
  const issues = [];
  if (!PROFILE_ID.test(profile.id)) {
    issues.push({ field: 'id', severity: 'error', message: 'Use 1-128 letters, numbers, dots, underscores, or hyphens.' });
  }
  if (!profile.name || profile.name.length > 256) {
    issues.push({ field: 'name', severity: 'error', message: 'Name is required and must be at most 256 characters. The backend also enforces a 256-byte UTF-8 limit.' });
  }
  if (!Number.isInteger(profile.port) || profile.port < 1 || profile.port > 65535) {
    issues.push({ field: 'port', severity: 'error', message: 'Port must be between 1 and 65535.' });
  }
  if (!Number.isInteger(profile.memoryMib) || profile.memoryMib < 128) {
    issues.push({ field: 'memoryMib', severity: 'error', message: 'Allocate at least 128 MiB.' });
  }
  if (!['loopback', 'lan'].includes(profile.networkScope)) {
    issues.push({ field: 'networkScope', severity: 'error', message: 'Choose loopback or LAN exposure.' });
  }
  if (profile.networkScope === 'lan') {
    issues.push({ field: 'networkScope', severity: 'warning', message: 'LAN exposure makes this server reachable by other devices on your local network.' });
  }
  if ((snapshot.profiles ?? []).some((candidate) => candidate.id === profile.id && candidate.id !== originalId)) {
    issues.push({ field: 'id', severity: 'error', message: 'A saved profile already uses this ID.' });
  }
  if ((snapshot.profiles ?? []).some((candidate) => candidate.port === profile.port && candidate.id !== originalId && candidate.enabled)) {
    issues.push({ field: 'port', severity: 'warning', message: 'Another enabled profile already reserves this port.' });
  }
  const availability = availabilityFor(profile.runtime, snapshot.runtimes ?? []);
  if (!availability.available) {
    issues.push({ field: 'runtime', severity: 'warning', message: `${runtimeLabel(profile.runtime)} is configuration-only: ${availability.reason}` });
  }
  return issues;
}

export function nextAvailablePort(profiles = [], preferred = 8080) {
  const used = new Set(profiles.map((profile) => Number(profile.port)));
  let port = Number(preferred) || 8080;
  for (let checked = 0; checked < 65535; checked += 1) {
    if (!used.has(port)) return port;
    port = port === 65535 ? 1 : port + 1;
  }
  throw new Error('No free profile port is available.');
}

export function nextProfileId(profiles = [], prefix = 'server') {
  let index = 1;
  const ids = new Set(profiles.map((profile) => profile.id));
  while (ids.has(`${prefix}-${index}`)) index += 1;
  return `${prefix}-${index}`;
}

export function cloneIdentity(profile, profiles = []) {
  let id = `${profile.id}-copy`;
  let index = 2;
  const ids = new Set(profiles.map((candidate) => candidate.id));
  while (ids.has(id)) id = `${profile.id}-copy-${index++}`;
  return { id, name: `${profile.name} copy` };
}

export function profileCanBeEdited(profile, snapshot = {}) {
  const server = (snapshot.servers ?? []).find((candidate) => candidate.serverId === profile.id);
  return !isActiveState(server?.state);
}
