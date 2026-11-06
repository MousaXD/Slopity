// JSON configuration only: no archives, executable file contents, commands or automatic starts.
export const TRANSFER_FORMAT = 'slopity-profiles';
export const TRANSFER_VERSION = 1;
export const MAX_IMPORT_BYTES = 256 * 1024;
export const MAX_IMPORT_PROFILES = 100;

const RUNTIMES = new Set(['built-in-http', 'java', 'node-js', 'python', 'php', 'native', 'custom']);
const KEYS = new Set(['id', 'name', 'runtime', 'networkScope', 'port', 'memoryMib',
  'enabled', 'executable', 'workingDirectory', 'arguments']);

function reject(message) {
  throw new Error(message);
}

function validateProfileData(value, index, strict = true) {
  const label = `Profile ${index + 1}`;
  if (!value || typeof value !== 'object' || Array.isArray(value)) reject(`${label} must be an object.`);
  if (strict && Object.keys(value).some((key) => !KEYS.has(key))) {
    reject(`${label} contains unsupported fields; only profile configuration data is accepted.`);
  }
  if (typeof value.id !== 'string' || !/^[A-Za-z0-9._-]{1,128}$/.test(value.id)) {
    reject(`${label} needs a valid 1-128 character profile ID.`);
  }
  if (typeof value.name !== 'string' || !value.name.trim() || new TextEncoder().encode(value.name).length > 256) {
    reject(`${label} needs a nonempty name of at most 256 UTF-8 bytes.`);
  }
  if (!RUNTIMES.has(value.runtime)) reject(`${label} uses an unknown runtime.`);
  if (value.networkScope !== 'loopback' && value.networkScope !== 'lan') reject(`${label} has an invalid network scope.`);
  if (!Number.isInteger(value.port) || value.port < 1 || value.port > 65535) reject(`${label} has an invalid port.`);
  if (!Number.isInteger(value.memoryMib) || value.memoryMib < 128 || value.memoryMib > 32768) {
    reject(`${label} has an invalid memory budget (128-32768 MiB).`);
  }
  if (typeof value.enabled !== 'boolean') reject(`${label} needs an enabled boolean.`);
  const strings = ['executable', 'workingDirectory'];
  for (const key of strings) {
    if (value[key] !== undefined && value[key] !== null &&
      (typeof value[key] !== 'string' || value[key].length > 2048)) {
      reject(`${label} has invalid ${key} metadata.`);
    }
  }
  const args = value.arguments ?? [];
  if (!Array.isArray(args) || args.length > 64 ||
    args.some((arg) => typeof arg !== 'string' || arg.length > 2048)) {
    reject(`${label} has invalid structured argument metadata.`);
  }
  if (value.runtime === 'built-in-http' &&
    (value.executable || value.workingDirectory || args.length)) {
    reject(`${label} cannot attach external executables to the built-in HTTP runtime.`);
  }

  return {
    id: value.id,
    name: value.name,
    runtime: value.runtime,
    networkScope: value.networkScope,
    port: value.port,
    memoryMib: value.memoryMib,
    enabled: value.enabled,
    executable: value.runtime === 'built-in-http' ? null : value.executable ?? null,
    workingDirectory: value.runtime === 'built-in-http' ? null : value.workingDirectory ?? null,
    arguments: value.runtime === 'built-in-http' ? [] : [...args],
  };
}

export function serializeProfiles(profiles = []) {
  if (!Array.isArray(profiles) || profiles.length > MAX_IMPORT_PROFILES) {
    reject(`Export supports at most ${MAX_IMPORT_PROFILES} profiles at a time.`);
  }
  const config = profiles.map((profile, index) => validateProfileData(profile, index, false));
  return JSON.stringify({ format: TRANSFER_FORMAT, version: TRANSFER_VERSION, profiles: config }, null, 2) + '\n';
}

export function parseProfileImport(text, existingProfiles = []) {
  if (typeof text !== 'string' || new TextEncoder().encode(text).length > MAX_IMPORT_BYTES) {
    reject('The selected JSON file exceeds the 256 KiB import limit.');
  }
  let parsed;
  try {
    parsed = JSON.parse(text);
  } catch {
    reject('The selected file is not valid JSON.');
  }
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed) ||
    Object.keys(parsed).some((key) => !['format', 'version', 'profiles'].includes(key)) ||
    parsed.format !== TRANSFER_FORMAT || parsed.version !== TRANSFER_VERSION ||
    !Array.isArray(parsed.profiles) || parsed.profiles.length > MAX_IMPORT_PROFILES) {
    reject('Unsupported profile file. Expected Slopity JSON format version 1 with at most 100 profiles.');
  }
  const existing = new Set(existingProfiles.map((profile) => profile.id));
  const seen = new Set();
  const entries = parsed.profiles.map((raw, index) => {
    const profile = validateProfileData(raw, index);
    const reason = existing.has(profile.id)
      ? 'Profile ID already exists on this device; will be skipped.'
      : seen.has(profile.id)
        ? 'Duplicate ID in this file; will be skipped.'
        : null;
    seen.add(profile.id);
    // Imported configurations are always disabled; never start or reserve an imported runtime.
    return { profile: { ...profile, enabled: false }, reason, previouslyEnabled: profile.enabled };
  });
  return {
    entries,
    importable: entries.filter((entry) => !entry.reason).length,
    skipped: entries.filter((entry) => entry.reason).length,
  };
}
