export function createStore(initial = {}) {
  let value = {
    snapshot: null,
    busy: false,
    refreshError: null,
    notice: null,
    selectedProfileId: null,
    editingProfileId: null,
    ...initial,
  };
  const listeners = new Set();

  return {
    get: () => value,
    set(patch) {
      value = { ...value, ...patch };
      for (const listener of listeners) listener(value);
      return value;
    },
    update(updater) {
      return this.set(updater(value));
    },
    subscribe(listener) {
      listeners.add(listener);
      listener(value);
      return () => listeners.delete(listener);
    },
  };
}
