const tauriInvoke = window.__TAURI_INTERNALS__?.invoke;

const elements = {
  platform: document.querySelector('#platform-pill'),
  hostTitle: document.querySelector('#host-title'),
  hostState: document.querySelector('#host-state'),
  hostReason: document.querySelector('#host-reason'),
  memoryBudget: document.querySelector('#memory-budget'),
  memoryNote: document.querySelector('#memory-note'),
  runtimeCount: document.querySelector('#runtime-count'),
  profileSummary: document.querySelector('#profile-summary'),
  profiles: document.querySelector('#profiles'),
  refresh: document.querySelector('#refresh'),
  newProfile: document.querySelector('#new-profile'),
  notice: document.querySelector('#notice'),
  editor: document.querySelector('#editor'),
  editorTitle: document.querySelector('#editor-title'),
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
};

const state = {
  profiles: [],
  servers: [],
  editingId: null,
  schemaVersion: null,
  busy: false,
};

async function loadDashboard({ announce = true } = {}) {
  try {
    requireNativeBridge();
    const snapshot = await tauriInvoke('dashboard_snapshot');
    renderDashboard(snapshot);
    if (announce) {
      showNotice('Profile and server state refreshed.', 'success');
    }
  } catch (error) {
    elements.hostTitle.textContent = 'Native bridge unavailable';
    elements.hostState.textContent = 'Preview only';
    elements.hostState.className = 'status blocked';
    elements.hostReason.textContent = String(error);
    elements.profiles.innerHTML = '<article class="card error">Run this frontend through the Tauri shell to read Rust state.</article>';
    if (announce) {
      showNotice(String(error), 'error');
    }
  }
}

function renderDashboard(snapshot) {
  state.profiles = snapshot.profiles;
  state.servers = snapshot.servers ?? [];
  state.schemaVersion = snapshot.profileSchemaVersion;

  elements.platform.textContent = `${snapshot.platform} · ${snapshot.architecture}`;
  if (snapshot.hostService.platform === 'android' && snapshot.hostService.foregroundServiceAvailable) {
    elements.hostTitle.textContent = 'Android foreground bridge compiled';
    elements.hostState.textContent = 'Device proof pending';
    elements.hostState.className = 'status neutral';
  } else if (snapshot.hostService.durableHostingAvailable) {
    elements.hostTitle.textContent = 'Built-in hosting available';
    elements.hostState.textContent = 'Ready';
    elements.hostState.className = 'status ready';
  } else {
    elements.hostTitle.textContent = 'Hosting proof still required';
    elements.hostState.textContent = 'Not claimed';
    elements.hostState.className = 'status blocked';
  }
  elements.hostReason.textContent = snapshot.hostService.reason;

  const budget = snapshot.resourcePlan.safeServerBudgetMib;
  elements.memoryBudget.textContent = budget > 0 ? `${budget} MiB` : 'Probe pending';
  elements.memoryNote.textContent = snapshot.resourcePlan.warning
    ?? 'The current shell does not yet include a platform memory probe.';

  const ready = snapshot.runtimes.filter((runtime) => runtime.available).length;
  elements.runtimeCount.textContent = `${ready} ready / ${snapshot.runtimes.length} known`;
  renderProfiles();
}

function renderProfiles() {
  const enabled = state.profiles.filter((profile) => profile.enabled).length;
  const active = state.servers.filter((server) => isActiveState(server.state)).length;
  elements.profileSummary.textContent = `${state.profiles.length} stored · ${enabled} enabled · ${active} active · schema v${state.schemaVersion ?? '?'}`;

  if (state.profiles.length === 0) {
    elements.profiles.innerHTML = '<article class="card empty-state"><h3>No profiles yet</h3><p class="muted">Create a built-in HTTP profile to host the first local probe.</p></article>';
    return;
  }

  elements.profiles.replaceChildren(...state.profiles.map(profileCard));
  setBusy(state.busy);
}

function profileCard(profile) {
  const card = document.createElement('article');
  card.className = 'card profile-card';
  const runtime = profile.runtime.replaceAll('-', ' ');
  const builtIn = profile.runtime === 'built-in-http';
  const server = serverFor(profile.id);
  const observedState = server?.state ?? 'stopped';
  const active = isActiveState(observedState);
  const stateClass = observedState === 'running'
    ? 'ready'
    : observedState === 'failed'
      ? 'blocked'
      : 'neutral';
  const runtimeText = builtIn
    ? 'Built-in Rust runtime'
    : profile.executable
      ? 'Executable configured, not verified'
      : 'Runtime path pending';
  const urls = server?.urls?.length
    ? `<div class="server-urls">${server.urls.map((url) => `<code>${escapeHtml(url)}</code>`).join('')}</div>`
    : '';
  const logs = server?.logs?.length
    ? `<details class="server-logs"><summary>Logs · ${server.requestCount} requests</summary><pre>${escapeHtml(server.logs.slice(-8).map((entry) => `[${entry.level}] ${entry.message}`).join('\n'))}</pre></details>`
    : '';
  const serverControl = builtIn
    ? active
      ? `<button class="server-stop" type="button" data-action="server-stop" data-id="${escapeHtml(profile.id)}">Stop</button>`
      : `<button class="primary" type="button" data-action="server-start" data-id="${escapeHtml(profile.id)}" data-locked="${profile.enabled ? 'false' : 'true'}" title="${profile.enabled ? 'Start the built-in HTTP server' : 'Enable the profile before starting'}">Start</button>`
    : '';

  card.innerHTML = `
    <div class="card-heading profile-heading">
      <div>
        <p class="eyebrow">${escapeHtml(runtime)}</p>
        <h3>${escapeHtml(profile.name)}</h3>
      </div>
      <span class="status ${stateClass}">${escapeHtml(observedState)}</span>
    </div>
    <p class="muted">${escapeHtml(runtimeText)} · ID ${escapeHtml(profile.id)}</p>
    <div class="profile-meta">
      <span>Port ${profile.port}</span>
      <span>${profile.memoryMib} MiB</span>
      <span>${escapeHtml(profile.networkScope)}</span>
      <span>${profile.enabled ? 'enabled' : 'disabled'}</span>
    </div>
    ${server?.lastError ? `<p class="server-error">${escapeHtml(server.lastError)}</p>` : ''}
    ${urls}
    ${logs}
    <div class="profile-actions">
      ${serverControl}
      <button type="button" data-action="edit" data-id="${escapeHtml(profile.id)}" data-locked="${active}">Edit</button>
      <button type="button" data-action="clone" data-id="${escapeHtml(profile.id)}">Clone</button>
      <button type="button" data-action="toggle" data-id="${escapeHtml(profile.id)}" data-locked="${active}">${profile.enabled ? 'Disable' : 'Enable'}</button>
      <button class="danger" type="button" data-action="delete" data-id="${escapeHtml(profile.id)}" data-locked="${active}">Delete</button>
    </div>
  `;
  return card;
}

function openEditor(profile = null) {
  state.editingId = profile?.id ?? null;
  elements.editorTitle.textContent = profile ? `Edit ${profile.name}` : 'Create profile';
  elements.id.readOnly = Boolean(profile);
  elements.id.value = profile?.id ?? nextProfileId();
  elements.name.value = profile?.name ?? 'My HTTP server';
  elements.runtime.value = profile?.runtime ?? 'built-in-http';
  elements.network.value = profile?.networkScope ?? 'loopback';
  elements.port.value = String(profile?.port ?? nextAvailablePort());
  elements.memory.value = String(profile?.memoryMib ?? 128);
  elements.executable.value = profile?.executable ?? '';
  elements.directory.value = profile?.workingDirectory ?? '';
  elements.arguments.value = profile?.arguments?.join('\n') ?? '';
  elements.enabled.checked = profile?.enabled ?? true;
  updateRuntimeFields();
  elements.editor.hidden = false;
  elements.editor.scrollIntoView({ behavior: 'smooth', block: 'start' });
  elements.name.focus();
}

function closeEditor() {
  state.editingId = null;
  elements.editor.hidden = true;
  elements.form.reset();
}

function updateRuntimeFields() {
  const builtIn = elements.runtime.value === 'built-in-http';
  [elements.executable, elements.directory, elements.arguments].forEach((field) => {
    field.disabled = builtIn;
  });
}

function readProfileForm() {
  const builtIn = elements.runtime.value === 'built-in-http';
  return {
    id: elements.id.value.trim(),
    name: elements.name.value.trim(),
    runtime: elements.runtime.value,
    executable: builtIn ? null : nullable(elements.executable.value),
    arguments: builtIn
      ? []
      : elements.arguments.value
        .split('\n')
        .map((argument) => argument.trim())
        .filter(Boolean),
    workingDirectory: builtIn ? null : nullable(elements.directory.value),
    port: Number(elements.port.value),
    memoryMib: Number(elements.memory.value),
    networkScope: elements.network.value,
    enabled: elements.enabled.checked,
  };
}

async function saveProfile(event) {
  event.preventDefault();
  const profile = readProfileForm();
  const command = state.editingId ? 'update_server_profile' : 'create_server_profile';
  const success = await runProfileCommand(command, { profile }, `${profile.name} saved.`);
  if (success) {
    closeEditor();
  }
}

async function handleProfileAction(event) {
  const button = event.target.closest('button[data-action]');
  if (!button || state.busy || button.disabled) {
    return;
  }

  const profile = state.profiles.find((candidate) => candidate.id === button.dataset.id);
  if (!profile) {
    showNotice('That profile no longer exists. Refreshing storage.', 'error');
    await loadDashboard();
    return;
  }

  switch (button.dataset.action) {
    case 'server-start':
      await runServerCommand(
        'start_builtin_http_server',
        { id: profile.id },
        `${profile.name} started.`,
      );
      break;
    case 'server-stop':
      await runServerCommand(
        'stop_builtin_http_server',
        { id: profile.id },
        `${profile.name} stopped.`,
      );
      break;
    case 'edit':
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
      if (window.confirm(`Delete ${profile.name}? This removes configuration only.`)) {
        await runProfileCommand(
          'delete_server_profile',
          { id: profile.id },
          `${profile.name} deleted.`,
        );
      }
      break;
    default:
      showNotice('Unknown profile action.', 'error');
  }
}

async function cloneProfile(profile) {
  const suggestedId = uniqueCloneId(profile.id);
  const newId = window.prompt('ID for the cloned profile', suggestedId)?.trim();
  if (!newId) {
    return;
  }
  const newName = window.prompt('Name for the cloned profile', `${profile.name} copy`)?.trim();
  if (!newName) {
    return;
  }

  await runProfileCommand(
    'clone_server_profile',
    { sourceId: profile.id, newId, newName },
    `${newName} cloned in a disabled state with a free port.`,
  );
}

async function runProfileCommand(command, args, successMessage) {
  try {
    requireNativeBridge();
    setBusy(true);
    state.profiles = await tauriInvoke(command, args);
    renderProfiles();
    showNotice(successMessage, 'success');
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
    requireNativeBridge();
    setBusy(true);
    const snapshot = await tauriInvoke(command, args);
    upsertServer(snapshot);
    renderProfiles();
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
  if (!tauriInvoke || state.busy) {
    return;
  }
  try {
    state.servers = await tauriInvoke('list_builtin_http_servers');
    renderProfiles();
  } catch (error) {
    console.error('Server state refresh failed', error);
  }
}

function upsertServer(snapshot) {
  const index = state.servers.findIndex((server) => server.serverId === snapshot.serverId);
  if (index === -1) {
    state.servers.push(snapshot);
  } else {
    state.servers[index] = snapshot;
  }
}

function serverFor(profileId) {
  return state.servers.find((server) => server.serverId === profileId) ?? null;
}

function isActiveState(serverState) {
  return ['starting', 'running', 'stopping'].includes(serverState);
}

function setBusy(busy) {
  state.busy = busy;
  elements.saveProfile.disabled = busy;
  elements.refresh.disabled = busy;
  elements.newProfile.disabled = busy;
  elements.profiles.querySelectorAll('button').forEach((button) => {
    button.disabled = busy || button.dataset.locked === 'true';
  });
}

function showNotice(message, kind) {
  elements.notice.hidden = false;
  elements.notice.className = `notice ${kind}`;
  elements.notice.textContent = message;
}

function requireNativeBridge() {
  if (!tauriInvoke) {
    throw new Error('The native Tauri bridge is not available in this browser preview.');
  }
}

function nextProfileId() {
  let index = state.profiles.length + 1;
  while (state.profiles.some((profile) => profile.id === `server-${index}`)) {
    index += 1;
  }
  return `server-${index}`;
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

function nextAvailablePort() {
  const used = new Set(state.profiles.map((profile) => profile.port));
  let port = 8_080;
  while (used.has(port) && port < 65_535) {
    port += 1;
  }
  return port;
}

function nullable(value) {
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

function escapeHtml(value) {
  return String(value)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#039;');
}

elements.refresh.addEventListener('click', () => loadDashboard());
elements.newProfile.addEventListener('click', () => openEditor());
elements.cancelEditor.addEventListener('click', closeEditor);
elements.runtime.addEventListener('change', updateRuntimeFields);
elements.form.addEventListener('submit', saveProfile);
elements.profiles.addEventListener('click', handleProfileAction);
loadDashboard();
window.setInterval(refreshServerStates, 2_000);
