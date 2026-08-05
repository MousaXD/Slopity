import { previewInvoke } from './preview.js';
import { createOverlayController } from './overlay.js';
import { createCatalogController } from './catalog.js';
import { createProfileEditor } from './profile-editor.js';
import { createProfileActions } from './profile-actions.js';
import { renderBridgeError, renderProfiles, renderTemplates } from './views.js';

const nativeInvoke = window.__TAURI_INTERNALS__?.invoke;
const previewMode = !nativeInvoke && new URLSearchParams(window.location.search).get('preview') === '1';
const tauriInvoke = nativeInvoke ?? (previewMode ? previewInvoke : null);

const elements = {
  platformLabel: document.querySelector('#platform-label'),
  profileSummary: document.querySelector('#profile-summary'),
  profiles: document.querySelector('#profiles'),
  refresh: document.querySelector('#refresh'),
  drawerRefresh: document.querySelector('#drawer-refresh'),
  notice: document.querySelector('#notice'),
  openDrawer: document.querySelector('#open-drawer'),
  closeDrawer: document.querySelector('#close-drawer'),
  drawer: document.querySelector('#app-drawer'),
  drawerBackdrop: document.querySelector('#drawer-backdrop'),
  modalBackdrop: document.querySelector('#modal-backdrop'),
  addButton: document.querySelector('#add-server-button'),
  templateList: document.querySelector('#template-list'),
  detailsIcon: document.querySelector('#details-icon'),
  detailsTitle: document.querySelector('#details-title'),
  detailsSubtitle: document.querySelector('#details-subtitle'),
  detailsContent: document.querySelector('#details-content'),
  detailsActions: document.querySelector('#details-actions'),
  editorSheet: document.querySelector('#editor-sheet'),
  editorTitle: document.querySelector('#editor-title'),
  editorSubtitle: document.querySelector('#editor-subtitle'),
  templateNote: document.querySelector('#template-note'),
  editorValidation: document.querySelector('#editor-validation'),
  cancelEditor: document.querySelector('#cancel-editor'),
  form: document.querySelector('#profile-form'),
  id: document.querySelector('#profile-id'),
  name: document.querySelector('#profile-name'),
  runtime: document.querySelector('#profile-runtime'),
  network: document.querySelector('#profile-network'),
  port: document.querySelector('#profile-port'),
  memory: document.querySelector('#profile-memory'),
  executable: document.querySelector('#profile-executable'),
  directory: document.querySelector('#profile-directory'),
  arguments: document.querySelector('#profile-arguments'),
  enabled: document.querySelector('#profile-enabled'),
  saveProfile: document.querySelector('#save-profile'),
  infoTitle: document.querySelector('#info-title'),
  infoSubtitle: document.querySelector('#info-subtitle'),
  infoContent: document.querySelector('#info-content'),
  infoActions: document.querySelector('#info-actions'),
  cardMenu: document.querySelector('#card-menu'),
};

const state = {
  snapshot: null,
  profiles: [],
  servers: [],
  schemaVersion: null,
  editingId: null,
  selectedProfileId: null,
  menuProfileId: null,
  currentSheet: null,
  sheetReturnFocus: null,
  drawerReturnFocus: null,
  busy: false,
  validationTimer: null,
};

let actions;
const overlays = createOverlayController({
  elements,
  state,
  closeCardMenu: () => actions?.closeCardMenu(),
});
let editor;
const catalog = createCatalogController({
  elements,
  state,
  openEditor: (...args) => editor.openEditor(...args),
  openSheet: overlays.openSheet,
  closeSheet: overlays.closeSheet,
  showNotice,
  nextAvailablePort,
});
editor = createProfileEditor({
  elements,
  state,
  overlays,
  previewMode,
  tauriInvoke,
  showNotice,
  nextAvailablePort,
  nextProfileId,
  runProfileCommand: (...args) => actions.runProfileCommand(...args),
});
actions = createProfileActions({
  elements,
  state,
  overlays,
  previewMode,
  tauriInvoke,
  showNotice,
  loadDashboard,
  renderProfileGrid,
  setBusy,
  profileById,
  serverFor,
  openEditor: editor.openEditor,
  loadProfileValidation: editor.loadProfileValidation,
  validationSlot,
});

async function loadDashboard({ announce = false } = {}) {
  try {
    requireNativeBridge();
    setBusy(true);
    applySnapshot(await tauriInvoke('dashboard_snapshot'));
    if (previewMode) {
      showNotice('Visual browser preview only. Native persistence and runtimes are not connected.', 'success');
    } else if (announce) {
      showNotice('Saved profiles and runtime state refreshed.', 'success');
    }
  } catch (error) {
    state.profiles = [];
    state.servers = [];
    renderBridgeError(elements, error);
    if (announce) {
      showNotice(String(error), 'error');
    }
  } finally {
    setBusy(false);
  }
}

function applySnapshot(snapshot) {
  state.snapshot = snapshot;
  state.profiles = Array.isArray(snapshot.profiles) ? snapshot.profiles : [];
  state.servers = Array.isArray(snapshot.servers) ? snapshot.servers : [];
  state.schemaVersion = snapshot.profileSchemaVersion;
  elements.platformLabel.textContent = `${snapshot.platform} · ${snapshot.architecture}`;
  renderProfileGrid();
  renderTemplates(elements, catalog.selectTemplate);
  actions.refreshOpenDetails();
}

function renderProfileGrid() {
  renderProfiles({
    elements,
    profiles: state.profiles,
    servers: state.servers,
    schemaVersion: state.schemaVersion,
    busy: state.busy,
    onAdd: openAddSheet,
    onDetails: actions.openDetails,
    onMenu: actions.openCardMenu,
  });
  applyBusyState();
}

function openAddSheet() {
  overlays.closeDrawer();
  actions.closeCardMenu();
  overlays.openSheet('add-sheet', elements.addButton);
}

function handleDrawerAction(event) {
  const button = event.target.closest('button[data-drawer-action]');
  if (!button) {
    return;
  }
  overlays.closeDrawer({ restoreFocus: false });
  switch (button.dataset.drawerAction) {
    case 'servers':
      document.querySelector('#saved-servers')?.scrollIntoView({ behavior: 'smooth', block: 'start' });
      elements.openDrawer.focus();
      break;
    case 'add':
      openAddSheet();
      break;
    case 'runtime':
      catalog.openRuntimeSupport();
      break;
    case 'device':
      catalog.openDeviceStatus();
      break;
    case 'settings':
      catalog.openInfoSheet({
        title: 'Settings',
        subtitle: 'Planned screen',
        paragraphs: ['Settings are not implemented yet. Slopity currently keeps profile configuration inside each saved server.'],
      });
      break;
    case 'about':
      catalog.openInfoSheet({
        title: 'About Slopity',
        subtitle: 'A portable server-control foundation',
        paragraphs: [
          'Slopity uses a Rust core, Tauri 2 shell, durable profile storage, and a shared static interface for Linux and Android.',
          'Only the fixed built-in HTTP probe is currently runnable. External runtime providers remain deliberately unavailable until they are implemented and proven.',
        ],
      });
      break;
    default:
      showNotice('Unknown navigation item.', 'error');
  }
}

function setBusy(busy) {
  state.busy = busy;
  applyBusyState();
}

function applyBusyState() {
  elements.refresh.disabled = state.busy;
  elements.drawerRefresh.disabled = state.busy;
  elements.saveProfile.disabled = state.busy || previewMode;
  document.querySelectorAll('[data-mutation="true"]').forEach((button) => {
    button.disabled = state.busy || previewMode || button.dataset.locked === 'true';
  });
  elements.profiles.setAttribute('aria-busy', String(state.busy));
}

function showNotice(message, kind) {
  elements.notice.hidden = false;
  elements.notice.className = `notice ${kind}`;
  elements.notice.textContent = message;
}

function requireNativeBridge() {
  if (!tauriInvoke) {
    throw new Error('The native Tauri bridge is not available. Add ?preview=1 for a visual-only browser preview.');
  }
}

function profileById(profileId) {
  return state.profiles.find((profile) => profile.id === profileId) ?? null;
}

function serverFor(profileId) {
  return state.servers.find((server) => server.serverId === profileId) ?? null;
}

function validationSlot() {
  return elements.detailsContent.querySelector('[data-validation-slot]');
}

function nextProfileId() {
  let index = state.profiles.length + 1;
  while (state.profiles.some((profile) => profile.id === `server-${index}`)) {
    index += 1;
  }
  return `server-${index}`;
}

function nextAvailablePort(preferred) {
  const used = new Set(state.profiles.map((profile) => profile.port));
  let port = preferred;
  while (used.has(port) && port < 65_535) {
    port += 1;
  }
  return port;
}

elements.openDrawer.addEventListener('click', overlays.openDrawer);
elements.closeDrawer.addEventListener('click', () => overlays.closeDrawer());
elements.drawerBackdrop.addEventListener('click', () => overlays.closeDrawer());
elements.drawer.addEventListener('click', handleDrawerAction);
elements.addButton.addEventListener('click', openAddSheet);
elements.refresh.addEventListener('click', () => loadDashboard({ announce: true }));
elements.drawerRefresh.addEventListener('click', async () => {
  overlays.closeDrawer();
  await loadDashboard({ announce: true });
});
elements.cancelEditor.addEventListener('click', editor.closeEditor);
elements.runtime.addEventListener('change', editor.updateRuntimeFields);
elements.form.addEventListener('input', editor.scheduleEditorValidation);
elements.form.addEventListener('submit', editor.saveProfile);
elements.detailsActions.addEventListener('click', actions.handleDetailAction);
elements.detailsContent.addEventListener('click', (event) => {
  const button = event.target.closest('button[data-copy-url]');
  if (button) {
    actions.copyText(button.dataset.copyUrl);
  }
});
elements.cardMenu.addEventListener('click', actions.handleCardMenuAction);
elements.modalBackdrop.addEventListener('click', () => overlays.closeSheet(state.currentSheet));
document.querySelectorAll('[data-close-sheet]').forEach((button) => {
  button.addEventListener('click', () => overlays.closeSheet(button.dataset.closeSheet));
});
document.addEventListener('keydown', overlays.handleKeydown);
document.addEventListener('pointerdown', (event) => {
  if (!elements.cardMenu.hidden && !elements.cardMenu.contains(event.target) && !event.target.closest('.card-menu-button')) {
    actions.closeCardMenu();
  }
});
window.addEventListener('resize', actions.closeCardMenu);
window.addEventListener('scroll', actions.closeCardMenu, { passive: true });

loadDashboard();
window.setInterval(actions.refreshServerStates, 2_000);
