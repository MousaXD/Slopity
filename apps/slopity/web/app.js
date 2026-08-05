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
  editingId: null,
  schemaVersion: null,
  busy: false,
};

async function loadDashboard() {
  try {
    requireNativeBridge();
    const snapshot = await tauriInvoke('dashboard_snapshot');
    renderDashboard(snapshot);
    showNotice('Profile storage refreshed.', 'success');
  } catch (error) {
    elements.hostTitle.textContent = 'Native bridge unavailable';
    elements.hostState.textContent = 'Preview only';
    elements.hostState.className = 'status blocked';
    elements.hostReason.textContent = String(error);
    elements.profiles.innerHTML = '<article class="card error">Run this frontend through the Tauri shell to read Rust state.</article>';
    showNotice(String(error), 'error');
  }
}

function renderDashboard(snapshot) {
  state.profiles = snapshot.profiles;
  state.schemaVersion = snapshot.profileSchemaVersion;

  elements.platform.textContent = `${snapshot.platform} · ${snapshot.architecture}`;
  elements.hostTitle.textContent = snapshot.hostService.durableHostingAvailable
    ? 'Desktop hosting boundary ready'
    : 'Hosting proof still required';
  elements.hostState.textContent = snapshot.hostService.durableHostingAvailable ? 'Foundation ready' : 'Not claimed';
  elements.hostState.className = snapshot.hostService.durableHostingAvailable ? 'status ready' : 'status blocked';
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
  elements.profileSummary.textContent = `${state.profiles.length} stored · ${enabled} enabled · schema v${state.schemaVersion ?? '?'}`;

  if (state.profiles.length === 0) {
    elements.profiles.innerHTML = '<article class="card empty-state"><h3>No profiles yet</h3><p class="muted">Create the first durable workload configuration. Nothing will execute.</p></article>';
    return;
  }

  elements.profiles.replaceChildren(...state.profiles.map(profileCard));
}

function profileCard(profile) {
  const card = document.createElement('article');
  card.className = 'card profile-card';
  const runtime = profile.runtime.replaceAll('-', ' ');
  const configured = profile.executable ? 'Executable configured' : 'Runtime path pending';
  const enabledClass = profile.enabled ? 'ready' : 'neutral';
  const enabledText = profile.enabled ? 'Enabled' : 'Disabled';
  card.innerHTML = `
    <div class="card-heading profile-heading">
      <div>
        <p class="eyebrow">${escapeHtml(runtime)}</p>
        <h3>${escapeHtml(profile.name)}</h3>
      </div>
      <span class="status ${enabledClass}">${enabledText}</span>
    </div>
    <p class="muted">${escapeHtml(configured)} · ID ${escapeHtml(profile.id)}</p>
    <div class="profile-meta">
      <span>Port ${profile.port}</span>
      <span>${profile.memoryMib} MiB</span>
      <span>${escapeHtml(profile.networkScope)}</span>
    </div>
    <div class="profile-actions">
      <button type="button" data-action="edit" data-id="${escapeHtml(profile.id)}">Edit</button>
      <button type="button" data-action="clone" data-id="${escapeHtml(profile.id)}">Clone</button>
      <button type="button" data-action="toggle" data-id="${escapeHtml(profile.id)}">${profile.enabled ? 'Disable' : 'Enable'}</button>
      <button class="danger" type="button" data-action="delete" data-id="${escapeHtml(profile.id)}">Delete</button>
    </div>
  `;
  return card;
}

function openEditor(profile = null) {
  state.editingId = profile?.id ?? null;
  elements.editorTitle.textContent = profile ? `Edit ${profile.name}` : 'Create profile';
  elements.id.readOnly = Boolean(profile);
  elements.id.value = profile?.id ?? nextProfileId();
  elements.name.value = profile?.name ?? 'New server';
  elements.runtime.value = profile?.runtime ?? 'native';
  elements.network.value = profile?.networkScope ?? 'loopback';
  elements.port.value = String(profile?.port ?? nextAvailablePort());
  elements.memory.value = String(profile?.memoryMib ?? 512);
  elements.executable.value = profile?.executable ?? '';
  elements.directory.value = profile?.workingDirectory ?? '';
  elements.arguments.value = profile?.arguments?.join('\n') ?? '';
  elements.enabled.checked = profile?.enabled ?? false;
  elements.editor.hidden = false;
  elements.editor.scrollIntoView({ behavior: 'smooth', block: 'start' });
  elements.name.focus();
}

function closeEditor() {
  state.editingId = null;
  elements.editor.hidden = true;
  elements.form.reset();
}

function readProfileForm() {
  return {
    id: elements.id.value.trim(),
    name: elements.name.value.trim(),
    runtime: elements.runtime.value,
    executable: nullable(elements.executable.value),
    arguments: elements.arguments.value
      .split('\n')
      .map((argument) => argument.trim())
      .filter(Boolean),
    workingDirectory: nullable(elements.directory.value),
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
  if (!button || state.busy) {
    return;
  }

  const profile = state.profiles.find((candidate) => candidate.id === button.dataset.id);
  if (!profile) {
    showNotice('That profile no longer exists. Refreshing storage.', 'error');
    await loadDashboard();
    return;
  }

  switch (button.dataset.action) {
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
    `${newName} cloned in a disabled state.`,
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

function setBusy(busy) {
  state.busy = busy;
  elements.saveProfile.disabled = busy;
  elements.refresh.disabled = busy;
  elements.newProfile.disabled = busy;
  elements.profiles.querySelectorAll('button').forEach((button) => {
    button.disabled = busy;
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

elements.refresh.addEventListener('click', loadDashboard);
elements.newProfile.addEventListener('click', () => openEditor());
elements.cancelEditor.addEventListener('click', closeEditor);
elements.form.addEventListener('submit', saveProfile);
elements.profiles.addEventListener('click', handleProfileAction);
loadDashboard();
