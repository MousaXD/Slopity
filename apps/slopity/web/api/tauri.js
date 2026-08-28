export function createSlopityApi(invoke = globalThis.window?.__TAURI_INTERNALS__?.invoke) {
  function requireInvoke() {
    if (typeof invoke !== 'function') {
      throw new Error('The native Tauri bridge is unavailable. Run Slopity through the desktop or Android app shell.');
    }
    return invoke;
  }

  return {
    dashboard: () => requireInvoke()('dashboard_snapshot'),
    validateProfile: (profile) => requireInvoke()('validate_server_profile', { profile }),
    createProfile: (profile) => requireInvoke()('create_server_profile', { profile }),
    updateProfile: (profile) => requireInvoke()('update_server_profile', { profile }),
    deleteProfile: (id) => requireInvoke()('delete_server_profile', { id }),
    setProfileEnabled: (id, enabled) => requireInvoke()('set_server_profile_enabled', { id, enabled }),
    cloneProfile: (sourceId, newId, newName) => requireInvoke()('clone_server_profile', { sourceId, newId, newName }),
    startServer: (id) => requireInvoke()('start_builtin_http_server', { id }),
    stopServer: (id) => requireInvoke()('stop_builtin_http_server', { id }),
    listServers: () => requireInvoke()('list_builtin_http_servers'),
  };
}
