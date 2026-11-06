export const ACTIVE_STATES = new Set(['starting', 'running', 'stopping']);

const RUNTIME_LABELS = {
  'built-in-http': 'Built-in HTTP',
  java: 'Java',
  'node-js': 'Node.js',
  python: 'Python',
  php: 'PHP',
  native: 'Native',
  custom: 'Custom',
};

export function runtimeLabel(runtime) {
  return RUNTIME_LABELS[runtime] ?? String(runtime || 'Unknown');
}

export function isActiveState(state) {
  return ACTIVE_STATES.has(state);
}

export function serverFor(profileId, servers = []) {
  return servers.find((server) => server.serverId === profileId) ?? null;
}

export function availabilityFor(runtime, runtimes = []) {
  return runtimes.find((entry) => entry.runtime === runtime) ?? {
    runtime,
    available: false,
    reason: 'Runtime availability has not been reported by the backend.',
  };
}

export function profileLifecycle(profile, snapshot = {}) {
  const server = serverFor(profile.id, snapshot.servers ?? []);
  const availability = availabilityFor(profile.runtime, snapshot.runtimes ?? []);
  const active = isActiveState(server?.state);
  const state = server?.state ?? 'stopped';
  const runnable = Boolean(profile.enabled && availability.available && !active);

  return {
    server,
    availability,
    active,
    state,
    runnable,
    canStop: active,
    startDisabledReason: active
      ? 'Server is already active.'
      : !profile.enabled
        ? 'Enable this profile before starting it.'
        : !availability.available
          ? availability.reason || 'Runtime is unavailable.'
          : null,
  };
}

export function commandErrorMessage(error) {
  if (typeof error === 'string') return error;
  if (error instanceof Error) return error.message || String(error);
  if (!error || typeof error !== 'object') return String(error ?? 'Unknown backend error.');

  if (error.kind === 'admission' && error.rejection) {
    const reasons = Array.isArray(error.rejection.reasons) ? error.rejection.reasons : [];
    const messages = reasons
      .map((reason) => String(reason?.message ?? '').trim())
      .filter(Boolean);
    if (messages.length) return `Start blocked: ${messages.join(' ')}`;
    return 'Start blocked by backend admission policy.';
  }

  if (typeof error.message === 'string' && error.message.trim()) return error.message.trim();

  try {
    const serialized = JSON.stringify(error);
    return serialized && serialized !== '{}' ? serialized : 'Unknown backend error.';
  } catch {
    return 'Unknown backend error.';
  }
}

export function titleCase(value) {
  return String(value ?? '')
    .replaceAll('-', ' ')
    .replace(/\b\w/g, (character) => character.toUpperCase());
}

export function runtimeBadge(profile, snapshot = {}) {
  const lifecycle = profileLifecycle(profile, snapshot);
  if (!lifecycle.availability.available) {
    return { key: 'unavailable', label: 'Runtime unavailable' };
  }
  return { key: lifecycle.state, label: titleCase(lifecycle.state) };
}
