import { createSlopityApi } from './api/tauri.js';
import { createStore } from './state/store.js';
import { renderDashboard } from './views/dashboard.js';
import { renderDeviceStatus } from './views/device-status.js';
import { createProfileEditor } from './views/profile-editor.js';
import { renderServerDetails } from './views/server-details.js';
import { closeDialog, createConfirmController, showDialog } from './views/dialogs.js';
import { clientValidateProfile, cloneIdentity, emptyDraft, nextAvailablePort, nextProfileId, profileToDraft } from './domain/profile.js';
import { availabilityFor, commandErrorMessage, isActiveState, profileLifecycle } from './domain/runtime.js';
import { MAX_IMPORT_BYTES, parseProfileImport, serializeProfiles } from './domain/profile-transfer.js';
import { renderImportPreview } from './views/import-preview.js';

const api = createSlopityApi();
const store = createStore();
const elements = collectElements(document);
const editor = createProfileEditor(document, elements.editorDialog);
const confirmAction = createConfirmController(elements.confirmDialog);
let pollTimer = null;
let serverPollPending = false;
let editorSaving = false;
let pendingImport = null;
let filters = { query: '', runtime: 'all', status: 'all', sort: 'name' };

function collectElements(root) {
  return {
    refresh: root.querySelector('#refresh-button'),
    create: root.querySelector('#create-button'),
    notice: root.querySelector('#global-notice'),
    connection: root.querySelector('#connection-status'),
    connectionMessage: root.querySelector('#connection-message'),
    retry: root.querySelector('#retry-button'),
    search: root.querySelector('#profile-search'),
    runtimeFilter: root.querySelector('#profile-runtime-filter'),
    statusFilter: root.querySelector('#profile-status-filter'),
    sort: root.querySelector('#profile-sort'),
    resetFilters: root.querySelector('#profile-filter-reset'),
    resultCount: root.querySelector('#profile-result-count'),
    import: root.querySelector('#import-profiles-button'),
    export: root.querySelector('#export-profiles-button'),
    importFile: root.querySelector('#import-profile-file'),
    importDialog: root.querySelector('#import-dialog'),
    importPreview: root.querySelector('#import-preview'),
    importConfirm: root.querySelector('#import-confirm'),
    editorSave: root.querySelector('#profile-form button[type="submit"]'),
    metrics: root.querySelector('#metric-grid'),
    runtimeGrid: root.querySelector('#runtime-grid'),
    resourceWarnings: root.querySelector('#resource-warnings'),
    recoveryNotices: root.querySelector('#recovery-notices'),
    serverGrid: root.querySelector('#server-grid'),
    deviceGrid: root.querySelector('#device-grid'),
    hostStatus: root.querySelector('#host-status'),
    deviceSubtitle: root.querySelector('#device-subtitle'),
    editorDialog: root.querySelector('#editor-dialog'),
    detailsDialog: root.querySelector('#details-dialog'),
    detailsContent: root.querySelector('#details-content'),
    detailsStartStop: root.querySelector('#details-start-stop'),
    detailsEdit: root.querySelector('#details-edit'),
    detailsEnable: root.querySelector('#details-enable'),
    detailsClone: root.querySelector('#details-clone'),
    detailsDelete: root.querySelector('#details-delete'),
    confirmDialog: root.querySelector('#confirm-dialog'),
    navButtons: [...root.querySelectorAll('[data-nav-target]')],
    sections: [...root.querySelectorAll('[data-section]')],
  };
}

function currentSnapshot() {
  return store.get().snapshot;
}

function currentProfile(id) {
  return currentSnapshot()?.profiles?.find((profile) => profile.id === id) ?? null;
}

function setNotice(message, kind = 'info') {
  store.set({ notice: message ? { message: String(message), kind } : null });
}

function renderState(state) {
  const snapshot = state.snapshot;
  elements.refresh.disabled = state.busy;
  elements.retry.disabled = state.busy;
  elements.create.disabled = state.busy || !snapshot;
  elements.import.disabled = state.busy || !snapshot;
  elements.export.disabled = state.busy || !snapshot?.profiles?.length;
  elements.connection.hidden = Boolean(snapshot && !state.refreshError);
  elements.connectionMessage.textContent = state.refreshError
    ? `${snapshot ? 'Backend connection lost; showing the last saved snapshot. ' : 'Cannot reach the native backend. '}${state.refreshError}`
    : 'Connecting to the native backend…';
  if (state.notice) {
    elements.notice.hidden = false;
    elements.notice.className = `global-notice notice-${state.notice.kind}`;
    elements.notice.textContent = state.notice.message;
    elements.notice.setAttribute('role', state.notice.kind === 'error' ? 'alert' : 'status');
  } else {
    elements.notice.hidden = true;
    elements.notice.textContent = '';
  }
  if (!snapshot) return;
  renderDashboard(document, elements, snapshot, {
    onStart: startProfile,
    onStop: stopProfile,
    onDetails: openDetails,
  }, filters);
  renderDeviceStatus(document, elements.deviceGrid, snapshot);
  elements.deviceSubtitle.textContent = `${snapshot.platform} · ${snapshot.architecture} · telemetry values remain unavailable when the backend cannot prove them`;
}

store.subscribe(renderState);

async function refreshDashboard({ announce = false } = {}) {
  try {
    store.set({ busy: true, refreshError: null });
    const snapshot = await api.dashboard();
    store.set({ snapshot, busy: false });
    if (announce) setNotice('Dashboard refreshed from the backend.', 'success');
    refreshOpenDetails();
    return snapshot;
  } catch (error) {
    store.set({ busy: false, refreshError: commandErrorMessage(error) });
    setNotice(`Dashboard refresh failed: ${commandErrorMessage(error)}`, 'error');
    throw error;
  }
}

async function refreshServerStates() {
  if (serverPollPending || document.hidden || store.get().busy || !currentSnapshot()) return;
  serverPollPending = true;
  try {
    const servers = await api.listServers();
    if (store.get().busy || !currentSnapshot()) return;
    store.set({ snapshot: { ...currentSnapshot(), servers }, refreshError: null });
    refreshOpenDetails();
  } catch (error) {
    if (!store.get().refreshError) {
      const message = commandErrorMessage(error);
      store.set({ refreshError: message });
      setNotice(`Server state refresh failed: ${message}`, 'error');
    }
  } finally {
    serverPollPending = false;
  }
}

async function withMutation(action, successMessage) {
  if (store.get().busy) return false;
  try {
    store.set({ busy: true });
    await action();
    const snapshot = await api.dashboard();
    store.set({ snapshot, busy: false, refreshError: null });
    setNotice(successMessage, 'success');
    refreshOpenDetails();
    return true;
  } catch (error) {
    store.set({ busy: false });
    setNotice(commandErrorMessage(error), 'error');
    refreshOpenDetails();
    return false;
  }
}

async function startProfile(profile) {
  const lifecycle = profileLifecycle(profile, currentSnapshot());
  if (!lifecycle.runnable) {
    setNotice(lifecycle.startDisabledReason || 'This profile cannot be started.', 'warning');
    return false;
  }
  return withMutation(() => api.startServer(profile.id), `${profile.name} started.`);
}

async function stopProfile(profile) {
  if (!isActiveState(profileLifecycle(profile, currentSnapshot()).server?.state)) {
    setNotice(`${profile.name} is not active.`, 'warning');
    return false;
  }
  return withMutation(() => api.stopServer(profile.id), `${profile.name} stopped.`);
}

function openCreate() {
  if (store.get().busy || !currentSnapshot()) {
    setNotice('Connect to the native backend before creating a profile.', 'warning');
    return;
  }
  const snapshot = currentSnapshot();
  const draft = emptyDraft(nextAvailablePort(snapshot.profiles, 8080));
  draft.id = nextProfileId(snapshot.profiles, 'http');
  editor.setDraft(draft, { runtimeAvailability: availabilityFor(draft.runtime, snapshot.runtimes) });
  store.set({ editingProfileId: null });
  showDialog(elements.editorDialog, document.querySelector('#profile-name'));
}

function openEdit(profile) {
  const lifecycle = profileLifecycle(profile, currentSnapshot());
  if (lifecycle.active) {
    setNotice('Stop the server before editing its saved profile.', 'warning');
    return;
  }
  editor.setDraft(profileToDraft(profile), {
    editing: true,
    runtimeAvailability: lifecycle.availability,
  });
  store.set({ editingProfileId: profile.id });
  showDialog(elements.editorDialog, document.querySelector('#profile-name'));
}

async function saveEditor(event) {
  event.preventDefault();
  if (editorSaving || store.get().busy) return;
  editorSaving = true;
  elements.editorSave.disabled = true;
  try {
  const snapshot = currentSnapshot() ?? { profiles: [], runtimes: [] };
  const editingId = store.get().editingProfileId;
  const profile = editor.readProfile();
  const clientIssues = clientValidateProfile(profile, snapshot, editingId);
  editor.renderIssues(clientIssues);
  const clientErrors = clientIssues.filter((issue) => issue.severity === 'error');
  if (clientErrors.length) {
    editor.focusFirstError(clientIssues);
    setNotice('Fix the highlighted profile fields before saving.', 'error');
    return;
  }

  let backendIssues;
  try {
    backendIssues = normalizeBackendIssues(await api.validateProfile(profile));
  } catch (error) {
    editor.renderIssues([{ severity: 'error', message: `Backend validation failed: ${commandErrorMessage(error)}` }]);
    setNotice(`Backend validation failed: ${commandErrorMessage(error)}`, 'error');
    return;
  }
  const allIssues = [...clientIssues, ...backendIssues];
  editor.renderIssues(allIssues);
  if (allIssues.some((issue) => issue.severity === 'error')) {
    editor.focusFirstError(allIssues);
    setNotice('The backend rejected this profile. Review the validation messages.', 'error');
    return;
  }

  const saved = await withMutation(
    () => (editingId ? api.updateProfile(profile) : api.createProfile(profile)),
    `${profile.name} saved.`,
  );
  if (saved) closeDialog(elements.editorDialog);
  } finally {
    editorSaving = false;
    elements.editorSave.disabled = false;
  }
}

function normalizeBackendIssues(issues = []) {
  if (!Array.isArray(issues)) {
    return [{ severity: 'error', message: 'Backend returned an invalid profile validation response.' }];
  }
  return issues.map((issue) => ({
    ...issue,
    field: issue.field || inferField(issue),
  }));
}

function inferField(issue = {}) {
  const code = String(issue.code ?? '').toLowerCase();
  const lower = String(issue.message ?? '').toLowerCase();
  if (code.startsWith('profile-id') || lower.includes('profile id')) return 'id';
  if (code.startsWith('profile-name') || lower.includes('profile name')) return 'name';
  if (code.includes('port') || lower.includes('port')) return 'port';
  if (code.includes('memory') || lower.includes('memory')) return 'memoryMib';
  if (code.includes('runtime') || lower.includes('runtime')) return 'runtime';
  if (code.includes('lan') || code.includes('network') || lower.includes('lan exposure') || lower.includes('loopback')) return 'networkScope';
  if (code.includes('executable')) return 'executable';
  if (code.includes('working-directory')) return 'workingDirectory';
  if (code.includes('argument')) return 'arguments';
  return null;
}

async function openDetails(profile) {
  store.set({ selectedProfileId: profile.id });
  let issues = [];
  try {
    issues = normalizeBackendIssues(await api.validateProfile(profile));
  } catch (error) {
    issues = [{ severity: 'warning', message: `Validation refresh failed: ${String(error)}` }];
  }
  const current = currentProfile(profile.id) ?? profile;
  renderServerDetails(document, elements.detailsContent, current, currentSnapshot(), issues);
  updateDetailsActions(current);
  showDialog(elements.detailsDialog, elements.detailsStartStop);
}

async function refreshOpenDetails() {
  if (!elements.detailsDialog.open) return;
  const id = store.get().selectedProfileId;
  const profile = currentProfile(id);
  if (!profile) {
    closeDialog(elements.detailsDialog);
    store.set({ selectedProfileId: null });
    return;
  }
  let issues = [];
  try {
    issues = normalizeBackendIssues(await api.validateProfile(profile));
  } catch (error) {
    issues = [{ severity: 'warning', message: `Validation refresh failed: ${String(error)}` }];
  }
  renderServerDetails(document, elements.detailsContent, profile, currentSnapshot(), issues);
  updateDetailsActions(profile);
}

function updateDetailsActions(profile) {
  const lifecycle = profileLifecycle(profile, currentSnapshot());
  elements.detailsStartStop.textContent = lifecycle.active ? 'Stop server' : lifecycle.availability.available ? 'Start server' : 'Runtime unavailable';
  elements.detailsStartStop.className = lifecycle.active ? 'button danger' : 'button primary';
  elements.detailsStartStop.disabled = store.get().busy || (lifecycle.active ? false : !lifecycle.runnable);
  elements.detailsStartStop.title = lifecycle.active ? 'Stop this server' : lifecycle.startDisabledReason;
  elements.detailsEdit.disabled = lifecycle.active || store.get().busy;
  elements.detailsEnable.disabled = lifecycle.active || store.get().busy;
  elements.detailsClone.disabled = store.get().busy;
  elements.detailsDelete.disabled = lifecycle.active || store.get().busy;
  elements.detailsEnable.textContent = profile.enabled ? 'Disable profile' : 'Enable profile';
}

async function toggleSelectedProfile() {
  const profile = currentProfile(store.get().selectedProfileId);
  if (!profile) return;
  await withMutation(
    () => api.setProfileEnabled(profile.id, !profile.enabled),
    `${profile.name} ${profile.enabled ? 'disabled' : 'enabled'}.`,
  );
}

async function cloneSelectedProfile() {
  const profile = currentProfile(store.get().selectedProfileId);
  if (!profile) return;
  const identity = cloneIdentity(profile, currentSnapshot().profiles);
  const ok = await withMutation(
    () => api.cloneProfile(profile.id, identity.id, identity.name),
    `${identity.name} cloned as a disabled profile.`,
  );
  if (ok) {
    const clone = currentProfile(identity.id);
    if (clone) openDetails(clone);
  }
}

async function deleteSelectedProfile() {
  const profile = currentProfile(store.get().selectedProfileId);
  if (!profile) return;
  const accepted = await confirmAction({
    heading: `Delete ${profile.name}?`,
    message: 'This removes the saved profile configuration. It does not delete external server files or data.',
    action: 'Delete profile',
    danger: true,
  });
  if (!accepted) return;
  const ok = await withMutation(() => api.deleteProfile(profile.id), `${profile.name} deleted.`);
  if (ok) {
    closeDialog(elements.detailsDialog);
    store.set({ selectedProfileId: null });
  }
}

function setupNavigation() {
  for (const button of elements.navButtons) {
    button.addEventListener('click', () => {
      const target = button.dataset.navTarget;
      for (const section of elements.sections) section.hidden = section.dataset.section !== target;
      for (const candidate of elements.navButtons) {
        if (candidate.classList.contains('nav-item')) {
          candidate.setAttribute('aria-current', candidate.dataset.navTarget === target ? 'page' : 'false');
        } else {
          candidate.removeAttribute('aria-current');
        }
      }
      document.querySelector(`[data-section="${target}"]`)?.focus({ preventScroll: true });
    });
  }
}

function setupProfileFilters() {
  const controls = [
    [elements.search, 'query', 'input'],
    [elements.runtimeFilter, 'runtime', 'change'],
    [elements.statusFilter, 'status', 'change'],
    [elements.sort, 'sort', 'change'],
  ];
  for (const [control, key, event] of controls) {
    control.addEventListener(event, () => {
      filters = { ...filters, [key]: control.value };
      renderState(store.get());
    });
  }
  elements.resetFilters.addEventListener('click', () => {
    filters = { query: '', runtime: 'all', status: 'all', sort: 'name' };
    for (const [control, key] of controls) control.value = filters[key];
    renderState(store.get());
    elements.search.focus();
  });
}

function exportSavedProfiles() {
  const profiles = currentSnapshot()?.profiles;
  if (!profiles?.length) return;
  try {
    const json = serializeProfiles(profiles);
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const download = document.createElement('a');
    download.href = url;
    download.download = 'slopity-profiles-v1.json';
    document.body.append(download);
    try {
      download.click();
    } finally {
      download.remove();
      window.setTimeout(() => URL.revokeObjectURL(url), 1000);
    }
    setNotice('JSON export requested. Check your app or browser downloads; some mobile WebViews do not support saving files.', 'info');
  } catch (error) {
    setNotice(`Could not export profiles: ${commandErrorMessage(error)}`, 'error');
  }
}

async function previewProfileFile(event) {
  const file = event.target.files?.[0];
  elements.importFile.value = '';
  if (!file) return;
  if (store.get().busy || !currentSnapshot()) return;
  try {
    if (file.size > MAX_IMPORT_BYTES) throw new Error('JSON imports must be 256 KiB or smaller.');
    const text = await file.text();
    const preview = parseProfileImport(text, currentSnapshot().profiles ?? []);
    pendingImport = preview;
    renderImportPreview(document, elements.importPreview, preview);
    elements.importConfirm.disabled = preview.importable === 0;
    showDialog(elements.importDialog, preview.importable ? elements.importConfirm : elements.importDialog.querySelector('[data-dialog-close]'));
  } catch (error) {
    pendingImport = null;
    setNotice(`Profile import rejected: ${commandErrorMessage(error)}`, 'error');
  }
}

async function importPreviewedProfiles() {
  if (!pendingImport || store.get().busy || !currentSnapshot()) return;
  const entries = pendingImport.entries.filter((entry) => !entry.reason);
  const existingIds = new Set(currentSnapshot().profiles.map((profile) => profile.id));
  let imported = 0;
  let skipped = pendingImport.skipped;
  const failures = [];
  store.set({ busy: true });
  elements.importConfirm.disabled = true;
  for (const { profile } of entries) {
    if (existingIds.has(profile.id)) {
      skipped += 1;
      continue;
    }
    try {
      const backendIssues = normalizeBackendIssues(await api.validateProfile(profile));
      const errors = backendIssues.filter((issue) => issue.severity === 'error');
      if (errors.length) throw new Error(errors.map((issue) => issue.message).join(' '));
      await api.createProfile(profile);
      existingIds.add(profile.id);
      imported += 1;
    } catch (error) {
      failures.push(`${profile.id}: ${commandErrorMessage(error)}`);
    }
  }

  let refreshFailure = null;
  try {
    store.set({ snapshot: await api.dashboard(), refreshError: null });
  } catch (error) {
    refreshFailure = commandErrorMessage(error);
    store.set({ refreshError: refreshFailure });
  }
  store.set({ busy: false });
  closeDialog(elements.importDialog);
  pendingImport = null;
  const summary = `Imported ${imported} disabled profile(s); skipped ${skipped} ID conflict(s); ${failures.length} backend failure(s).`;
  const details = [...failures.slice(0, 3), ...(refreshFailure ? [`Dashboard refresh failed: ${refreshFailure}`] : [])].join(' ');
  setNotice(`${summary} ${details}`.trim(), failures.length || refreshFailure || skipped ? 'warning' : 'success');
}

function startPolling() {
  stopPolling();
  pollTimer = window.setInterval(refreshServerStates, 2000);
}

function stopPolling() {
  if (pollTimer) window.clearInterval(pollTimer);
  pollTimer = null;
}

elements.refresh.addEventListener('click', () => refreshDashboard({ announce: true }).catch(() => {}));
elements.retry.addEventListener('click', () => refreshDashboard({ announce: true }).catch(() => {}));
elements.import.addEventListener('click', () => elements.importFile.click());
elements.importFile.addEventListener('change', previewProfileFile);
elements.importConfirm.addEventListener('click', importPreviewedProfiles);
elements.export.addEventListener('click', exportSavedProfiles);
elements.importDialog.addEventListener('close', () => {
  if (!store.get().busy) pendingImport = null;
});
elements.create.addEventListener('click', openCreate);
editor.form.addEventListener('submit', saveEditor);
elements.detailsStartStop.addEventListener('click', async () => {
  const profile = currentProfile(store.get().selectedProfileId);
  if (!profile) return;
  const lifecycle = profileLifecycle(profile, currentSnapshot());
  await (lifecycle.active ? stopProfile(profile) : startProfile(profile));
});
elements.detailsEdit.addEventListener('click', () => {
  const profile = currentProfile(store.get().selectedProfileId);
  if (!profile) return;
  closeDialog(elements.detailsDialog);
  openEdit(profile);
});
elements.detailsEnable.addEventListener('click', toggleSelectedProfile);
elements.detailsClone.addEventListener('click', cloneSelectedProfile);
elements.detailsDelete.addEventListener('click', deleteSelectedProfile);
for (const button of document.querySelectorAll('[data-dialog-close]')) {
  button.addEventListener('click', () => closeDialog(document.querySelector(`#${button.dataset.dialogClose}`)));
}
elements.detailsDialog.addEventListener('close', () => store.set({ selectedProfileId: null }));
elements.editorDialog.addEventListener('close', () => store.set({ editingProfileId: null }));
document.addEventListener('visibilitychange', () => {
  if (document.hidden) stopPolling();
  else {
    refreshDashboard().catch(() => {});
    startPolling();
  }
});

setupNavigation();
setupProfileFilters();
refreshDashboard().catch(() => {}).finally(startPolling);
