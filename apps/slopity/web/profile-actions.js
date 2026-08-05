import { isActiveState, node } from './dom.js';
import { renderDetails } from './views.js';

export function createProfileActions({
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
  openEditor,
  loadProfileValidation,
  validationSlot,
}) {
  async function openDetails(profileId) {
    const profile = profileById(profileId);
    if (!profile) {
      showNotice('That saved profile no longer exists. Refreshing.', 'error');
      await loadDashboard();
      return;
    }
    state.selectedProfileId = profileId;
    renderSelectedDetails(profile);
    overlays.openSheet('details-sheet');
    await loadProfileValidation(profile, validationSlot());
  }

  function renderSelectedDetails(profile) {
    renderDetails({
      elements,
      profile,
      server: serverFor(profile.id),
      busy: state.busy,
      previewMode,
    });
  }

  function openCardMenu(profile, anchor) {
    closeCardMenu();
    state.menuProfileId = profile.id;
    const active = isActiveState(serverFor(profile.id)?.state);
    const builtIn = profile.runtime === 'built-in-http';
    const items = [menuAction('Open details', 'details')];
    if (builtIn) {
      items.push(menuAction(active ? 'Stop server' : 'Start server', active ? 'stop' : 'start', {
        disabled: !active && !profile.enabled,
        lockText: !active && !profile.enabled ? 'Enable first' : '',
      }));
    }
    items.push(menuAction('Edit', 'edit', { disabled: active, lockText: active ? 'Stop first' : '' }));
    items.push(menuAction('Clone', 'clone'));
    items.push(menuAction(profile.enabled ? 'Disable' : 'Enable', 'toggle', {
      disabled: active,
      lockText: active ? 'Stop first' : '',
    }));
    items.push(menuAction('Delete', 'delete', {
      disabled: active,
      lockText: active ? 'Stop first' : '',
      danger: true,
    }));
    elements.cardMenu.replaceChildren(...items);
    elements.cardMenu.hidden = false;
    anchor.setAttribute('aria-expanded', 'true');
    anchor.dataset.menuAnchor = 'true';
    positionCardMenu(anchor, items.length);
    elements.cardMenu.querySelector('button:not(:disabled)')?.focus();
  }

  function menuAction(label, action, { disabled = false, lockText = '', danger = false } = {}) {
    const button = node('button', {
      className: danger ? 'danger-menu-item' : '',
      attrs: { type: 'button', role: 'menuitem' },
    }, [
      node('span', { text: label }),
      lockText ? node('span', { className: 'menu-lock', text: lockText }) : null,
    ]);
    button.dataset.menuAction = action;
    button.disabled = disabled || state.busy || previewMode;
    return button;
  }

  function positionCardMenu(anchor, itemCount) {
    const rect = anchor.getBoundingClientRect();
    const width = Math.min(228, window.innerWidth - 24);
    const height = itemCount * 47 + 12;
    elements.cardMenu.style.left = `${Math.min(window.innerWidth - width - 12, Math.max(12, rect.right - width))}px`;
    elements.cardMenu.style.top = `${rect.bottom + height > window.innerHeight - 12 ? Math.max(12, rect.top - height) : rect.bottom + 5}px`;
  }

  function closeCardMenu() {
    const anchor = document.querySelector('[data-menu-anchor="true"]');
    anchor?.setAttribute('aria-expanded', 'false');
    anchor?.removeAttribute('data-menu-anchor');
    elements.cardMenu.hidden = true;
    elements.cardMenu.replaceChildren();
    state.menuProfileId = null;
  }

  async function handleCardMenuAction(event) {
    const button = event.target.closest('button[data-menu-action]');
    if (!button || button.disabled) {
      return;
    }
    const profileId = state.menuProfileId;
    const action = button.dataset.menuAction;
    closeCardMenu();
    await runProfileAction(profileId, action);
  }

  async function handleDetailAction(event) {
    const button = event.target.closest('button[data-detail-action]');
    if (button && !button.disabled) {
      await runProfileAction(state.selectedProfileId, button.dataset.detailAction);
    }
  }

  async function runProfileAction(profileId, action) {
    const profile = profileById(profileId);
    if (!profile) {
      showNotice('That saved profile no longer exists. Refreshing.', 'error');
      await loadDashboard();
      return;
    }
    switch (action) {
      case 'details':
        await openDetails(profile.id);
        break;
      case 'start':
        await runServerCommand('start_builtin_http_server', { id: profile.id }, `${profile.name} started.`);
        break;
      case 'stop':
        await runServerCommand('stop_builtin_http_server', { id: profile.id }, `${profile.name} stopped.`);
        break;
      case 'edit':
        overlays.closeSheet('details-sheet', { restoreFocus: false });
        openEditor(profile);
        break;
      case 'clone':
        await cloneProfile(profile);
        break;
      case 'toggle':
        await runProfileCommand(
          'set_server_profile_enabled',
          { id: profile.id, enabled: !profile.enabled },
          `${profile.name} ${profile.enabled ? 'disabled' : 'enabled'}.`,
        );
        break;
      case 'delete':
        await deleteProfile(profile);
        break;
      default:
        showNotice('Unknown server action.', 'error');
    }
  }

  async function cloneProfile(profile) {
    if (previewMode) {
      showNotice('Profile writes are disabled in browser preview mode.', 'error');
      return;
    }
    const newId = window.prompt('ID for the cloned profile', uniqueCloneId(profile.id))?.trim();
    if (!newId) {
      return;
    }
    const newName = window.prompt('Name for the cloned profile', `${profile.name} copy`)?.trim();
    if (newName) {
      await runProfileCommand(
        'clone_server_profile',
        { sourceId: profile.id, newId, newName },
        `${newName} cloned in a disabled state with a free port.`,
      );
    }
  }

  async function deleteProfile(profile) {
    if (previewMode) {
      showNotice('Profile writes are disabled in browser preview mode.', 'error');
      return;
    }
    if (!window.confirm(`Delete ${profile.name}? This removes saved configuration only.`)) {
      return;
    }
    if (await runProfileCommand('delete_server_profile', { id: profile.id }, `${profile.name} deleted.`)) {
      overlays.closeSheet('details-sheet');
    }
  }

  async function runProfileCommand(command, args, successMessage) {
    try {
      setBusy(true);
      state.profiles = await tauriInvoke(command, args);
      renderProfileGrid();
      showNotice(successMessage, 'success');
      refreshOpenDetails();
      return true;
    } catch (error) {
      showNotice(String(error), 'error');
      return false;
    } finally {
      setBusy(false);
    }
  }

  async function runServerCommand(command, args, successMessage) {
    try {
      setBusy(true);
      const snapshot = await tauriInvoke(command, args);
      upsertServer(snapshot);
      renderProfileGrid();
      refreshOpenDetails();
      const urlNote = snapshot.urls?.length ? ` ${snapshot.urls.join(' · ')}` : '';
      showNotice(`${successMessage}${urlNote}`, 'success');
      return true;
    } catch (error) {
      showNotice(String(error), 'error');
      await refreshServerStates();
      return false;
    } finally {
      setBusy(false);
    }
  }

  async function refreshServerStates() {
    if (!tauriInvoke || state.busy || previewMode) {
      return;
    }
    try {
      state.servers = await tauriInvoke('list_builtin_http_servers');
      renderProfileGrid();
      refreshOpenDetails();
    } catch (error) {
      console.error('Server state refresh failed', error);
    }
  }

  function refreshOpenDetails() {
    if (state.currentSheet !== 'details-sheet') {
      return;
    }
    const profile = profileById(state.selectedProfileId);
    if (!profile) {
      overlays.closeSheet('details-sheet');
      return;
    }
    renderSelectedDetails(profile);
    loadProfileValidation(profile, validationSlot());
  }

  function upsertServer(snapshot) {
    const index = state.servers.findIndex((server) => server.serverId === snapshot.serverId);
    if (index === -1) {
      state.servers.push(snapshot);
    } else {
      state.servers[index] = snapshot;
    }
  }

  async function copyText(value) {
    try {
      await navigator.clipboard.writeText(value);
      showNotice('URL copied to clipboard.', 'success');
    } catch (error) {
      showNotice(`Could not copy the URL: ${error}`, 'error');
    }
  }

  function uniqueCloneId(sourceId) {
    let candidate = `${sourceId}-copy`;
    let index = 2;
    while (state.profiles.some((profile) => profile.id === candidate)) {
      candidate = `${sourceId}-copy-${index}`;
      index += 1;
    }
    return candidate;
  }

  return {
    closeCardMenu,
    copyText,
    handleCardMenuAction,
    handleDetailAction,
    openCardMenu,
    openDetails,
    refreshOpenDetails,
    refreshServerStates,
    runProfileCommand,
  };
}
