export const UI_PREFERENCES_KEY = 'slopity-ui-preferences-v1';
export const REFRESH_INTERVALS_MS = [2000, 5000, 10000, 30000];
export const DEFAULT_UI_PREFERENCES = Object.freeze({ autoRefresh: true, refreshIntervalMs: 2000 });

export function normalizeUiPreferences(value) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return { ...DEFAULT_UI_PREFERENCES };
  }
  return {
    autoRefresh: typeof value.autoRefresh === 'boolean'
      ? value.autoRefresh
      : DEFAULT_UI_PREFERENCES.autoRefresh,
    refreshIntervalMs: REFRESH_INTERVALS_MS.includes(value.refreshIntervalMs)
      ? value.refreshIntervalMs
      : DEFAULT_UI_PREFERENCES.refreshIntervalMs,
  };
}

export function loadUiPreferences(storage) {
  try {
    return normalizeUiPreferences(JSON.parse(storage?.getItem(UI_PREFERENCES_KEY) ?? 'null'));
  } catch {
    return { ...DEFAULT_UI_PREFERENCES };
  }
}

export function saveUiPreferences(storage, value) {
  const preferences = normalizeUiPreferences(value);
  try {
    if (!storage || typeof storage.setItem !== 'function') return { preferences, persisted: false };
    storage.setItem(UI_PREFERENCES_KEY, JSON.stringify(preferences));
    return { preferences, persisted: true };
  } catch {
    return { preferences, persisted: false };
  }
}
