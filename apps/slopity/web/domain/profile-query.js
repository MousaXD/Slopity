import { availabilityFor, isActiveState, serverFor } from './runtime.js';

const FILTERS = new Set(['all', 'active', 'stopped', 'disabled', 'unavailable', 'failed']);
const SORTS = new Set(['name', 'port', 'state']);

export function profileStatus(profile, snapshot = {}) {
  const server = serverFor(profile.id, snapshot.servers ?? []);
  if (isActiveState(server?.state)) return 'active';
  if (server?.state === 'failed') return 'failed';
  if (!profile.enabled) return 'disabled';
  if (!availabilityFor(profile.runtime, snapshot.runtimes ?? []).available) return 'unavailable';
  return 'stopped';
}

export function selectProfiles(snapshot = {}, options = {}) {
  const query = String(options.query ?? '').trim().toLocaleLowerCase();
  const runtime = String(options.runtime ?? 'all');
  const status = FILTERS.has(options.status) ? options.status : 'all';
  const sort = SORTS.has(options.sort) ? options.sort : 'name';
  const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' });
  return [...(snapshot.profiles ?? [])]
    .filter((profile) => {
      if (runtime !== 'all' && profile.runtime !== runtime) return false;
      if (status === 'unavailable' && availabilityFor(profile.runtime, snapshot.runtimes ?? []).available) return false;
      if (status !== 'all' && status !== 'unavailable' && profileStatus(profile, snapshot) !== status) return false;
      if (!query) return true;
      return [profile.name, profile.id, profile.runtime, String(profile.port)]
        .some((value) => String(value ?? '').toLocaleLowerCase().includes(query));
    })
    .sort((left, right) => {
      let result = 0;
      if (sort === 'port') result = left.port - right.port;
      if (sort === 'state') result = collator.compare(profileStatus(left, snapshot), profileStatus(right, snapshot));
      return result || collator.compare(left.name, right.name) || collator.compare(left.id, right.id);
    });
}
