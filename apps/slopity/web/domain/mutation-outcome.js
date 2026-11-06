// A successful mutation must not be reported as failed solely because a later refresh fails.

export function canStartServerPoll(state, hidden, pending) {
  return Boolean(!pending && !hidden && !state.busy && !state.refreshError && state.snapshot);
}

export function canApplyServerPoll(state, requestedSnapshot) {
  return Boolean(requestedSnapshot && !state.busy && !state.refreshError &&
    state.snapshot === requestedSnapshot);
}
export async function performBackendMutation(action, refreshDashboard) {
  try {
    await action();
  } catch (actionError) {
    return { committed: false, actionError, snapshot: null, refreshError: null };
  }
  try {
    const snapshot = await refreshDashboard();
    return { committed: true, actionError: null, snapshot, refreshError: null };
  } catch (refreshError) {
    return { committed: true, actionError: null, snapshot: null, refreshError };
  }
}
