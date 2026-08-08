const tauriInvoke = window.__TAURI_INTERNALS__?.invoke;

const elements = {
  menuButton: document.querySelector('#menu-button'),
  refreshButton: document.querySelector('#refresh-button'),
  addServerButton: document.querySelector('#add-server-button'),
  profiles: document.querySelector('#profiles'),
  summary: document.querySelector('#summary-strip'),
  notice: document.querySelector('#notice'),
  drawerOverlay: document.querySelector('#drawer-overlay'),
  drawer: document.querySelector('#drawer'),
  drawerNav: document.querySelector('#drawer-nav'),
  drawerPlatform: document.querySelector('#drawer-platform'),
  hostStateDot: document.querySelector('#host-state-dot'),
  hostStateCopy: document.querySelector('#host-state-copy'),
  addOverlay: document.querySelector('#add-overlay'),
  addSheet: document.querySelector('#add-sheet'),
  templateList: document.querySelector('#template-list'),
  detailsOverlay: document.querySelector('#details-overlay'),
  detailsSheet: document.querySelector('#details-sheet'),
  detailsContent: document.querySelector('#details-content'),
  actionsOverlay: document.querySelector('#actions-overlay'),
  actionsSheet: document.querySelector('#actions-sheet'),
  actionsTitle: document.querySelector('#actions-title'),
  actionsSubtitle: document.querySelector('#actions-subtitle'),
  actionsList: document.querySelector('#actions-list'),
  templateOverlay: document.querySelector('#template-overlay'),
  templateSheet: document.querySelector('#template-sheet'),
  templateSupportContent: document.querySelector('#template-support-content'),
  editorOverlay: document.querySelector('#editor-overlay'),
  editorSheet: document.querySelector('#editor-sheet'),
  editorTitle: document.querySelector('#editor-title'),
  editorNote: document.querySelector('#editor-note'),
  form: document.querySelector('#profile-form'),
  formValidation: document.querySelector('#form-validation'),
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
  externalFields: document.querySelector('#external-runtime-fields'),
  saveProfile: document.querySelector('#save-profile'),
};

const templates = [
  {
    id: 'minecraft',
    title: 'Minecraft Server',
    description: 'Prepare a Minecraft server profile',
    support: 'planned',
    supportLabel: 'Runtime support coming later',
    runtime: 'java',
    icon: 'minecraft',
    defaultName: 'Minecraft Server',
    preferredPort: 25_565,
    memoryMib: 2_048,
  },
  {
    id: 'website',
    title: 'Website',
    description: 'Create a web-hosting foundation',
    support: 'builtin-foundation',
    supportLabel: 'Uses the built-in HTTP probe today',
    runtime: 'built-in-http',
    icon: 'website',
    defaultName: 'My Website',
    preferredPort: 8_080,
    memoryMib: 128,
  },
  {
    id: 'import',
    title: 'Import Server',
    description: 'Import an existing server profile',
    support: 'planned-import',
    supportLabel: 'Import flow is not implemented yet',
    icon: 'import',
  },
  {
    id: 'node',
    title: 'Node.js App',
    description: 'Prepare a Node.js application profile',
    support: 'planned',
    supportLabel: 'Runtime support coming later',
    runtime: 'node-js',
    icon: 'node',
    defaultName: 'Node.js App',
    preferredPort: 3_000,
    memoryMib: 512,
  },
  {
    id: 'builtin',
    title: 'Built-in HTTP Server',
    description: 'Run Slopity’s safe built-in HTTP probe',
    support: 'ready',
    supportLabel: 'Available now',
    runtime: 'built-in-http',
    icon: 'builtin',
    defaultName: 'Built-in HTTP Server',
    preferredPort: 8_080,
    memoryMib: 128,
  },
  {
    id: 'custom',
    title: 'Custom Template',
    description: 'Start from a blank editable profile',
    support: 'editor',
    supportLabel: 'Configuration only until a runtime is verified',
    runtime: 'custom',
    icon: 'custom',
    defaultName: '',
    preferredPort: 8_080,
    memoryMib: 256,
  },
];

const state = {
  profiles: [],
  servers: [],
  runtimes: [],
  snapshot: null,
  schemaVersion: null,
  editingId: null,
  editingTemplate: null,
  detailsId: null,
  actionsId: null,
  busy: false,
  openLayers: [],
  focusOrigins: new Map(),
};

function node(tag, className = '', text = '') {
  const element = document.createElement(tag);
  if (className) element.className = className;
  if (text) element.textContent = text;
  return element;
}

function actionButton(text, className = 'secondary-action') {
  const button = node('button', className, text);
  button.type = 'button';
  return button;
}

function createSvg(commands, viewBox = '0 0 24 24') {
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('viewBox', viewBox);
  svg.setAttribute('aria-hidden', 'true');
  for (const [tag, attributes, text] of commands) {
    const child = document.createElementNS('http://www.w3.org/2000/svg', tag);
    for (const [name, value] of Object.entries(attributes)) child.setAttribute(name, value);
    if (text) child.textContent = text;
    svg.append(child);
  }
  return svg;
}

function createTemplateIcon(kind) {
  const wrapper = node('div', `template-icon ${kind}`);
  let svg;
  switch (kind) {
    case 'minecraft':
      svg = createSvg([
        ['path', { d: 'M4 7.5 12 4l8 3.5-8 3.7L4 7.5Z' }],
        ['path', { d: 'M4 7.5v9L12 20v-8.8L4 7.5Z' }],
        ['path', { d: 'M20 7.5v9L12 20v-8.8l8-3.7Z' }],
        ['path', { d: 'm7 6.2 3 1.3 3-1.4 3 1.3' }],
        ['path', { d: 'M7 11v2m3-.6v2.2m7-3.4v2.2m-3-.8v2.4' }],
      ]);
      break;
    case 'website':
      svg = createSvg([
        ['circle', { cx: '12', cy: '12', r: '8.5' }],
        ['path', { d: 'M3.8 12h16.4M12 3.5c2.2 2.3 3.3 5.1 3.3 8.5S14.2 18.2 12 20.5M12 3.5C9.8 5.8 8.7 8.6 8.7 12s1.1 6.2 3.3 8.5' }],
      ]);
      break;
    case 'import':
      svg = createSvg([
        ['path', { d: 'M7.5 17.5H6a4 4 0 0 1-.5-8A6.5 6.5 0 0 1 18 8.2a4.2 4.2 0 0 1 .2 8.3H16' }],
        ['path', { d: 'M12 10v9m-3-6 3-3 3 3' }],
      ]);
      break;
    case 'node':
      svg = createSvg([
        ['path', { d: 'm12 3 7 4v10l-7 4-7-4V7l7-4Z' }],
        ['text', { x: '12', y: '15.2', 'text-anchor': 'middle', 'font-size': '7.2', 'font-weight': '800', fill: 'currentColor', stroke: 'none' }, 'JS'],
      ]);
      break;
    case 'builtin':
      svg = createSvg([
        ['rect', { x: '4', y: '5', width: '16', height: '5', rx: '1.4' }],
        ['rect', { x: '4', y: '14', width: '16', height: '5', rx: '1.4' }],
        ['circle', { cx: '7', cy: '7.5', r: '0.7', fill: 'currentColor', stroke: 'none' }],
        ['circle', { cx: '7', cy: '16.5', r: '0.7', fill: 'currentColor', stroke: 'none' }],
        ['path', { d: 'M10 7.5h6M10 16.5h6' }],
      ]);
      break;
    case 'custom':
    case 'native':
      svg = createSvg([
        ['rect', { x: '4', y: '4', width: '16', height: '16', rx: '2' }],
        ['path', { d: 'm8 9 3 3-3 3m5 0h3' }],
      ]);
      break;
    default:
      svg = createSvg([
        ['rect', { x: '4', y: '5', width: '16', height: '5', rx: '1.4' }],
        ['rect', { x: '4', y: '14', width: '16', height: '5', rx: '1.4' }],
      ]);
  }
  wrapper.append(svg);
  return wrapper;
}

function renderTemplates() {
  elements.templateList.replaceChildren(...templates.map((template) => {
    const button = node('button', 'template-option');
    button.type = 'button';
    button.append(createTemplateIcon(template.icon));
    const copy = node('div');
    copy.append(node('h3', '', template.title), node('p', '', template.description), node('small', '', template.supportLabel));
    button.append(copy, node('span', 'chevron', '›'));
    button.addEventListener('click', () => selectTemplate(template));
    return button;
  }));
}

async function loadDashboard({ announce = false } = {}) {
  try {
    requireNativeBridge();
    renderDashboard(await tauriInvoke('dashboard_snapshot'));
    if (announce) showNotice('Server state refreshed.', 'success');
  } catch (error) {
    state.snapshot = null;
    state.profiles = [];
    state.servers = [];
    elements.summary.textContent = 'Native bridge unavailable';
    elements.profiles.replaceChildren(emptyState('Run Slopity through the Tauri shell', String(error)));
    if (announce) showNotice(String(error), 'error');
  }
}

function renderDashboard(snapshot) {
  state.snapshot = snapshot;
  state.profiles = snapshot.profiles ?? [];
  state.servers = snapshot.servers ?? [];
  state.runtimes = snapshot.runtimes ?? [];
  state.schemaVersion = snapshot.profileSchemaVersion;
  elements.drawerPlatform.textContent = `${snapshot.platform} · ${snapshot.architecture}`;
  renderHostCapability(snapshot.hostService);
  renderProfiles();
}

function renderHostCapability(capability) {
  elements.hostStateDot.className = 'host-dot';
  if (capability.platform === 'android' && capability.foregroundServiceAvailable) {
    elements.hostStateDot.classList.add('pending');
    elements.hostStateCopy.textContent = 'Android foreground service compiled; real-device background durability is still unproven.';
  } else if (capability.durableHostingAvailable) {
    elements.hostStateDot.classList.add('ready');
    elements.hostStateCopy.textContent = 'Built-in HTTP hosting capability is available on this platform.';
  } else {
    elements.hostStateDot.classList.add('pending');
    elements.hostStateCopy.textContent = capability.reason || 'Hosting capability proof is still pending.';
  }
}

function renderProfiles() {
  const active = state.servers.filter((server) => isActiveState(server.state)).length;
  const enabled = state.profiles.filter((profile) => profile.enabled).length;
  elements.summary.textContent = `${state.profiles.length} saved · ${enabled} enabled · ${active} active · schema v${state.schemaVersion ?? '?'}`;
  if (!state.profiles.length) {
    elements.profiles.replaceChildren(emptyState('No saved servers yet', 'Use Add Server to create a built-in HTTP profile or a clearly marked placeholder.'));
    return;
  }
  elements.profiles.replaceChildren(...state.profiles.map(profileCard));
}

function emptyState(title, copy) {
  const card = node('article', 'empty-card');
  card.append(node('h3', '', title), node('p', '', copy));
  return card;
}

function profileCard(profile) {
  const display = profileDisplay(profile);
  const status = profileStatus(profile);
  const card = node('article', 'server-card');
  card.tabIndex = 0;
  card.setAttribute('role', 'button');
  card.setAttribute('aria-label', `Open ${profile.name} details. Status: ${status.label}.`);
  card.append(createTemplateIcon(display.icon));
  const copy = node('div', 'server-card-content');
  copy.append(node('h3', '', profile.name), node('p', 'server-type', display.type), statusPill(status));
  card.append(copy);
  const menu = node('button', 'server-card-menu');
  menu.type = 'button';
  menu.setAttribute('aria-label', `Open actions for ${profile.name}`);
  const dots = node('span', 'kebab');
  dots.setAttribute('aria-hidden', 'true');
  dots.append(node('i'), node('i'), node('i'));
  menu.append(dots);
  menu.addEventListener('click', (event) => {
    event.stopPropagation();
    openActions(profile);
  });
  card.append(menu);
  card.addEventListener('click', () => openDetails(profile));
  card.addEventListener('keydown', (event) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      openDetails(profile);
    }
  });
  return card;
}

function profileDisplay(profile) {
  if (profile.runtime === 'built-in-http' && profile.id.startsWith('website-')) return { type: 'Website · built-in web probe', icon: 'website' };
  if (profile.runtime === 'built-in-http') return { type: 'Built-in HTTP Server', icon: 'builtin' };
  if (profile.runtime === 'java') return { type: 'Minecraft / Java Server', icon: 'minecraft' };
  if (profile.runtime === 'node-js') return { type: 'Node.js App', icon: 'node' };
  if (profile.runtime === 'native') return { type: 'Native profile · provider unavailable', icon: 'native' };
  if (profile.runtime === 'custom') return { type: 'Custom Template', icon: 'custom' };
  return { type: `${runtimeLabel(profile.runtime)} · provider unavailable`, icon: 'custom' };
}

function profileStatus(profile) {
  if (profile.runtime !== 'built-in-http') return { key: 'unavailable', label: 'Unavailable' };
  const server = serverFor(profile.id);
  return server?.state ? { key: server.state, label: titleCase(server.state) } : { key: 'stopped', label: 'Stopped' };
}

function statusPill(status) {
  return node('span', `status-pill status-${status.key}`, status.label);
}

function runtimeLabel(runtime) {
  return ({ 'built-in-http': 'Built-in HTTP', java: 'Java', 'node-js': 'Node.js', python: 'Python', php: 'PHP', native: 'Native', custom: 'Custom' })[runtime] ?? runtime;
}

function titleCase(value) {
  return String(value).replaceAll('-', ' ').replace(/\b\w/g, (character) => character.toUpperCase());
}

function serverFor(profileId) {
  return state.servers.find((server) => server.serverId === profileId) ?? null;
}

function isActiveState(serverState) {
  return ['starting', 'running', 'stopping'].includes(serverState);
}

function openDrawer() {
  setLayerOpen(elements.drawerOverlay, true, elements.drawer);
  elements.menuButton.setAttribute('aria-expanded', 'true');
}

function openAddServer() {
  closeLayer(elements.drawerOverlay);
  setLayerOpen(elements.addOverlay, true, elements.addSheet);
}

function actionRow(title, hint, handler, danger = false) {
  const button = node('button', `action-row${danger ? ' danger' : ''}`);
  button.type = 'button';
  const copy = node('span');
  copy.append(node('strong', '', title));
  if (hint) {
    const small = node('small', '', hint);
    small.style.display = 'block';
    small.style.marginTop = '3px';
    copy.append(small);
  }
  button.append(copy, node('span', 'chevron', '›'));
  button.addEventListener('click', handler);
  return button;
}

function openActions(profile) {
  state.actionsId = profile.id;
  elements.actionsSubtitle.textContent = profile.name;
  const server = serverFor(profile.id);
  const active = isActiveState(server?.state);
  const builtIn = profile.runtime === 'built-in-http';
  const rows = [actionRow('Open details', 'Runtime, URLs, logs and validation', () => {
    closeLayer(elements.actionsOverlay);
    openDetails(profile);
  })];
  if (builtIn) {
    const lifecycle = actionRow(active ? 'Stop server' : 'Start server', active ? 'Gracefully stop the built-in HTTP listener' : profile.enabled ? 'Start the built-in HTTP listener' : 'Enable this profile before starting', async () => {
      closeLayer(elements.actionsOverlay);
      await runServerCommand(active ? 'stop_builtin_http_server' : 'start_builtin_http_server', { id: profile.id }, `${profile.name} ${active ? 'stopped' : 'started'}.`);
    });
    lifecycle.disabled = !active && !profile.enabled;
    rows.push(lifecycle);
  }
  const edit = actionRow('Edit', active ? 'Stop the server before editing' : 'Edit saved configuration', () => {
    closeLayer(elements.actionsOverlay);
    openEditor(profile);
  });
  edit.disabled = active;
  rows.push(edit, actionRow('Clone', 'Create a disabled copy on the next free port', async () => {
    closeLayer(elements.actionsOverlay);
    await cloneProfile(profile);
  }));
  const toggle = actionRow(profile.enabled ? 'Disable' : 'Enable', active ? 'Stop the server before changing enabled state' : 'Change whether this configuration may be started', async () => {
    closeLayer(elements.actionsOverlay);
    await runProfileCommand('set_server_profile_enabled', { id: profile.id, enabled: !profile.enabled }, `${profile.name} ${profile.enabled ? 'disabled' : 'enabled'}.`);
  });
  toggle.disabled = active;
  const remove = actionRow('Delete', active ? 'Stop the server before deleting' : 'Delete this saved configuration', async () => {
    closeLayer(elements.actionsOverlay);
    await deleteProfile(profile);
  }, true);
  remove.disabled = active;
  rows.push(toggle, remove);
  elements.actionsList.replaceChildren(...rows);
  setLayerOpen(elements.actionsOverlay, true, elements.actionsSheet);
}

async function openDetails(profile) {
  state.detailsId = profile.id;
  setLayerOpen(elements.detailsOverlay, true, elements.detailsSheet);
  await renderDetails(profile);
}

function detailSection(title) {
  const section = node('section', 'detail-section');
  section.append(node('h3', '', title));
  return section;
}

function detailStats(profile, server) {
  const section = detailSection('Configuration and runtime');
  const grid = node('div', 'detail-grid');
  const stats = [
    ['Runtime', runtimeLabel(profile.runtime)], ['Port', String(profile.port)], ['Memory', `${profile.memoryMib} MiB`],
    ['Network', titleCase(profile.networkScope)], ['Configuration', profile.enabled ? 'Enabled' : 'Disabled'], ['Requests', String(server?.requestCount ?? 0)],
  ];
  if (server?.bindAddress) stats.push(['Bind address', server.bindAddress]);
  for (const [label, value] of stats) {
    const stat = node('div', 'detail-stat');
    stat.append(node('small', '', label), node('strong', '', value));
    grid.append(stat);
  }
  section.append(grid);
  return section;
}

async function renderDetails(profile) {
  if (state.detailsId !== profile.id || elements.detailsOverlay.hidden) return;
  const current = state.profiles.find((candidate) => candidate.id === profile.id) ?? profile;
  const display = profileDisplay(current);
  const status = profileStatus(current);
  const server = serverFor(current.id);
  const active = isActiveState(server?.state);
  const builtIn = current.runtime === 'built-in-http';
  let issues = [];
  try {
    if (tauriInvoke) issues = await tauriInvoke('validate_server_profile', { profile: current });
  } catch (error) {
    issues = [{ severity: 'warning', message: `Validation could not be refreshed: ${String(error)}` }];
  }
  if (state.detailsId !== current.id || elements.detailsOverlay.hidden) return;
  const content = document.createDocumentFragment();
  const heading = node('div', 'detail-heading');
  heading.append(createTemplateIcon(display.icon));
  const copy = node('div');
  copy.append(node('h2', '', current.name), node('p', '', display.type), statusPill(status));
  const close = node('button', 'icon-button sheet-close', '×');
  close.type = 'button';
  close.setAttribute('aria-label', 'Close server details');
  close.addEventListener('click', () => closeLayer(elements.detailsOverlay));
  heading.append(copy, close);
  content.append(heading);
  if (!builtIn) {
    content.append(node('div', 'support-copy', state.runtimes.find((runtime) => runtime.runtime === current.runtime)?.reason || 'This provider is not verified. The profile is saved configuration only and cannot be started from this dashboard.'));
  } else if (current.id.startsWith('website-')) {
    content.append(node('div', 'support-copy', 'Website currently maps to Slopity’s built-in HTTP probe. It proves local web hosting and lifecycle control, but does not yet serve a selected static folder or deploy a web application.'));
  }
  content.append(detailStats(current, server));
  if (server?.lastError) {
    const section = detailSection('Last runtime error');
    section.append(node('div', 'validation-item error', server.lastError));
    content.append(section);
  }
  const urlsSection = detailSection('Available URLs');
  const urls = node('div', 'url-list');
  if (server?.urls?.length) for (const url of server.urls) urls.append(node('div', 'url-item', url));
  else urls.append(node('p', 'empty-copy', builtIn ? 'Start this built-in HTTP server to expose its current URLs.' : 'No URLs are available because this runtime provider is unsupported.'));
  urlsSection.append(urls);
  content.append(urlsSection);
  const logsSection = detailSection('Recent runtime logs');
  const logs = node('div', 'log-list');
  const recent = server?.logs?.slice(-20) ?? [];
  if (recent.length) {
    for (const entry of recent) {
      const row = node('div', 'log-item');
      row.append(node('span', `log-level ${entry.level}`, entry.level), node('span', '', entry.message));
      logs.append(row);
    }
  } else logs.append(node('p', 'empty-copy', 'No runtime logs have been recorded for this server yet.'));
  logsSection.append(logs);
  content.append(logsSection);
  const validationSection = detailSection('Profile validation');
  const validation = node('div', 'validation-list');
  if (issues.length) for (const issue of issues) validation.append(node('div', `validation-item ${issue.severity}`, issue.message));
  else validation.append(node('div', 'validation-item', 'No validation issues are currently reported for this profile.'));
  validationSection.append(validation);
  content.append(validationSection);
  const management = detailSection('Profile actions');
  const manage = node('div', 'actions-list');
  const toggle = actionRow(current.enabled ? 'Disable profile' : 'Enable profile', active ? 'Stop the server first' : 'Update saved configuration state', () => runProfileCommand('set_server_profile_enabled', { id: current.id, enabled: !current.enabled }, `${current.name} ${current.enabled ? 'disabled' : 'enabled'}.`));
  toggle.disabled = active;
  const remove = actionRow('Delete profile', active ? 'Stop the server first' : 'Remove this saved configuration', () => deleteProfile(current), true);
  remove.disabled = active;
  manage.append(toggle, actionRow('Clone profile', 'Create a disabled copy on a free port', () => cloneProfile(current)), remove);
  management.append(manage);
  content.append(management);
  const controls = node('div', 'detail-actions');
  if (builtIn) {
    const lifecycle = actionButton(active ? 'Stop server' : 'Start server', active ? 'danger-action' : 'primary-action');
    lifecycle.disabled = !active && !current.enabled;
    lifecycle.addEventListener('click', () => runServerCommand(active ? 'stop_builtin_http_server' : 'start_builtin_http_server', { id: current.id }, `${current.name} ${active ? 'stopped' : 'started'}.`));
    controls.append(lifecycle);
  } else {
    const unsupported = actionButton('Runtime unavailable');
    unsupported.disabled = true;
    controls.append(unsupported);
  }
  const edit = actionButton('Edit profile');
  edit.disabled = active;
  edit.addEventListener('click', () => {
    closeLayer(elements.detailsOverlay);
    openEditor(current);
  });
  controls.append(edit);
  content.append(controls);
  elements.detailsContent.replaceChildren(content);
}

function selectTemplate(template) {
  closeLayer(elements.addOverlay);
  if (['ready', 'builtin-foundation', 'editor'].includes(template.support)) openEditor(null, template);
  else if (template.support === 'planned') openUnsupportedTemplate(template);
  else openImportPlaceholder(template);
}

function supportHeading(icon, title, subtitle) {
  const hero = node('div', 'support-hero');
  hero.append(createTemplateIcon(icon));
  const copy = node('div');
  copy.append(node('h2', '', title), node('p', '', subtitle));
  const close = node('button', 'icon-button sheet-close', '×');
  close.type = 'button';
  close.setAttribute('aria-label', `Close ${title}`);
  close.addEventListener('click', () => closeLayer(elements.templateOverlay));
  hero.append(copy, close);
  return hero;
}

function openUnsupportedTemplate(template) {
  const content = document.createDocumentFragment();
  content.append(supportHeading(template.icon, template.title, 'Runtime support is not operational yet.'));
  content.append(node('div', 'support-copy', `${template.title} can be saved as a disabled placeholder profile, but Slopity will not claim it can deploy or start this runtime. No executable, package, server archive, or external process will be launched.`));
  content.append(node('span', 'support-badge', 'Planned runtime · configuration only'));
  const create = actionButton('Create disabled placeholder', 'primary-action');
  create.style.width = '100%';
  create.style.marginTop = '22px';
  create.addEventListener('click', () => createPlaceholder(template));
  content.append(create);
  elements.templateSupportContent.replaceChildren(content);
  setLayerOpen(elements.templateOverlay, true, elements.templateSheet);
}

function openImportPlaceholder(template) {
  const content = document.createDocumentFragment();
  content.append(supportHeading(template.icon, 'Import Server', 'Import is planned, not simulated.'));
  content.append(node('div', 'support-copy', 'Slopity does not currently have a safe profile import/export format or a VPS import connector. This option intentionally does not invent remote access, credentials, file transfer, or a working import.'));
  content.append(node('span', 'support-badge', 'Planned feature · no remote import performed'));
  elements.templateSupportContent.replaceChildren(content);
  setLayerOpen(elements.templateOverlay, true, elements.templateSheet);
}

async function createPlaceholder(template) {
  const profile = {
    id: nextProfileId(template.id), name: template.defaultName, runtime: template.runtime, executable: null, arguments: [], workingDirectory: null,
    port: nextAvailablePort(template.preferredPort), memoryMib: template.memoryMib, networkScope: 'loopback', enabled: false,
  };
  if (await runProfileCommand('create_server_profile', { profile }, `${profile.name} saved as a disabled placeholder. Runtime support is still unavailable.`)) {
    closeLayer(elements.templateOverlay);
    const created = state.profiles.find((candidate) => candidate.id === profile.id);
    if (created) openDetails(created);
  }
}

function openEditor(profile = null, template = null) {
  state.editingId = profile?.id ?? null;
  state.editingTemplate = template?.id ?? null;
  elements.editorTitle.textContent = profile ? `Edit ${profile.name}` : template ? `Add ${template.title}` : 'Create profile';
  elements.editorNote.textContent = profile ? 'Update saved configuration. Running built-in servers must be stopped before editing.' : template?.id === 'website' ? 'Website currently uses the built-in HTTP probe. Static folder and web-app deployment are not implemented yet.' : template?.id === 'builtin' ? 'This creates a working profile for Slopity’s built-in Rust HTTP server.' : 'Blank configuration profile. External runtime support remains unavailable until a provider is verified.';
  elements.id.readOnly = Boolean(profile);
  const prefix = template?.id === 'website' ? 'website' : template?.id === 'builtin' ? 'http' : template?.id ?? 'server';
  elements.id.value = profile?.id ?? nextProfileId(prefix);
  elements.name.value = profile?.name ?? template?.defaultName ?? '';
  elements.runtime.value = profile?.runtime ?? template?.runtime ?? 'built-in-http';
  elements.network.value = profile?.networkScope ?? 'loopback';
  elements.port.value = String(profile?.port ?? nextAvailablePort(template?.preferredPort ?? 8_080));
  elements.memory.value = String(profile?.memoryMib ?? template?.memoryMib ?? 128);
  elements.executable.value = profile?.executable ?? '';
  elements.directory.value = profile?.workingDirectory ?? '';
  elements.arguments.value = profile?.arguments?.join('\n') ?? '';
  elements.enabled.checked = profile?.enabled ?? ((template?.runtime ?? 'built-in-http') === 'built-in-http');
  elements.formValidation.hidden = true;
  elements.formValidation.replaceChildren();
  updateRuntimeFields();
  setLayerOpen(elements.editorOverlay, true, elements.editorSheet, elements.name);
}

function updateRuntimeFields() {
  const builtIn = elements.runtime.value === 'built-in-http';
  elements.externalFields.hidden = builtIn;
  [elements.executable, elements.directory, elements.arguments].forEach((field) => { field.disabled = builtIn; });
  if (!builtIn && !state.editingId) elements.enabled.checked = false;
}

function readProfileForm() {
  const builtIn = elements.runtime.value === 'built-in-http';
  return {
    id: elements.id.value.trim(), name: elements.name.value.trim(), runtime: elements.runtime.value,
    executable: builtIn ? null : nullable(elements.executable.value),
    arguments: builtIn ? [] : elements.arguments.value.split('\n').map((argument) => argument.trim()).filter(Boolean),
    workingDirectory: builtIn ? null : nullable(elements.directory.value), port: Number(elements.port.value), memoryMib: Number(elements.memory.value),
    networkScope: elements.network.value, enabled: elements.enabled.checked,
  };
}

async function saveProfile(event) {
  event.preventDefault();
  if (state.busy) return;
  const profile = readProfileForm();
  try {
    requireNativeBridge();
    setBusy(true);
    const issues = await tauriInvoke('validate_server_profile', { profile });
    renderFormValidation(issues);
    if (issues.some((issue) => issue.severity === 'error')) {
      showNotice('Fix the profile validation errors before saving.', 'error');
      return;
    }
    state.profiles = await tauriInvoke(state.editingId ? 'update_server_profile' : 'create_server_profile', { profile });
    renderProfiles();
    closeLayer(elements.editorOverlay);
    showNotice(`${profile.name} saved.`, 'success');
  } catch (error) {
    showNotice(String(error), 'error');
    elements.formValidation.hidden = false;
    elements.formValidation.textContent = String(error);
  } finally {
    setBusy(false);
  }
}

function renderFormValidation(issues) {
  if (!issues.length) {
    elements.formValidation.hidden = true;
    elements.formValidation.replaceChildren();
    return;
  }
  elements.formValidation.replaceChildren(...issues.map((issue) => node('div', '', `${titleCase(issue.severity)}: ${issue.message}`)));
  elements.formValidation.hidden = false;
}

async function cloneProfile(profile) {
  const newId = window.prompt('ID for the cloned profile', uniqueCloneId(profile.id))?.trim();
  if (!newId) return;
  const newName = window.prompt('Name for the cloned profile', `${profile.name} copy`)?.trim();
  if (!newName) return;
  await runProfileCommand('clone_server_profile', { sourceId: profile.id, newId, newName }, `${newName} cloned in a disabled state with a free port.`);
}

async function deleteProfile(profile) {
  if (!window.confirm(`Delete ${profile.name}? This removes saved configuration only.`)) return;
  if (await runProfileCommand('delete_server_profile', { id: profile.id }, `${profile.name} deleted.`)) {
    if (state.detailsId === profile.id) closeLayer(elements.detailsOverlay);
  }
}

async function runProfileCommand(command, args, successMessage) {
  try {
    requireNativeBridge();
    setBusy(true);
    state.profiles = await tauriInvoke(command, args);
    renderProfiles();
    showNotice(successMessage, 'success');
    if (state.detailsId && !elements.detailsOverlay.hidden) {
      const current = state.profiles.find((profile) => profile.id === state.detailsId);
      if (current) await renderDetails(current);
    }
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
    showNotice(`${successMessage}${snapshot.urls?.length ? ` ${snapshot.urls.join(' · ')}` : ''}`, 'success');
    if (state.detailsId === snapshot.serverId && !elements.detailsOverlay.hidden) {
      const profile = state.profiles.find((candidate) => candidate.id === snapshot.serverId);
      if (profile) await renderDetails(profile);
    }
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
  if (!tauriInvoke || state.busy) return;
  try {
    state.servers = await tauriInvoke('list_builtin_http_servers');
    renderProfiles();
    if (state.detailsId && !elements.detailsOverlay.hidden) {
      const profile = state.profiles.find((candidate) => candidate.id === state.detailsId);
      if (profile) await renderDetails(profile);
    }
  } catch (error) {
    console.error('Server state refresh failed', error);
  }
}

function upsertServer(snapshot) {
  const index = state.servers.findIndex((server) => server.serverId === snapshot.serverId);
  if (index === -1) state.servers.push(snapshot);
  else state.servers[index] = snapshot;
}

function setBusy(busy) {
  state.busy = busy;
  elements.saveProfile.disabled = busy;
  elements.refreshButton.disabled = busy;
  elements.addServerButton.disabled = busy;
}

function showNotice(message, kind = 'info') {
  elements.notice.hidden = false;
  elements.notice.className = `notice ${kind}`;
  elements.notice.textContent = message;
}

function requireNativeBridge() {
  if (!tauriInvoke) throw new Error('The native Tauri bridge is not available in this browser preview.');
}

function nextProfileId(prefix = 'server') {
  let index = 1;
  while (state.profiles.some((profile) => profile.id === `${prefix}-${index}`)) index += 1;
  return `${prefix}-${index}`;
}

function uniqueCloneId(sourceId) {
  let candidate = `${sourceId}-copy`;
  let index = 2;
  while (state.profiles.some((profile) => profile.id === candidate)) candidate = `${sourceId}-copy-${index++}`;
  return candidate;
}

function nextAvailablePort(preferred = 8_080) {
  const used = new Set(state.profiles.map((profile) => profile.port));
  let port = Number(preferred) || 8_080;
  const start = port;
  do {
    if (!used.has(port)) return port;
    port = port === 65_535 ? 1 : port + 1;
  } while (port !== start);
  throw new Error('No free profile port is available.');
}

function nullable(value) {
  const trimmed = value.trim();
  return trimmed.length ? trimmed : null;
}

function setLayerOpen(overlay, open, focusTarget = null, explicitFocus = null) {
  if (!open) return closeLayer(overlay);
  if (!overlay.hidden) return;
  state.focusOrigins.set(overlay.id, document.activeElement);
  overlay.hidden = false;
  state.openLayers = state.openLayers.filter((id) => id !== overlay.id);
  state.openLayers.push(overlay.id);
  document.body.classList.add('modal-open');
  window.setTimeout(() => (explicitFocus || focusTarget || overlay.querySelector('button,input,select,textarea,[tabindex]:not([tabindex="-1"])'))?.focus(), 0);
}

function closeLayer(overlay, restoreFocus = true) {
  if (overlay.hidden) return;
  overlay.hidden = true;
  state.openLayers = state.openLayers.filter((id) => id !== overlay.id);
  if (overlay === elements.drawerOverlay) elements.menuButton.setAttribute('aria-expanded', 'false');
  if (overlay === elements.detailsOverlay) state.detailsId = null;
  if (overlay === elements.actionsOverlay) state.actionsId = null;
  if (overlay === elements.editorOverlay) {
    state.editingId = null;
    state.editingTemplate = null;
    elements.form.reset();
  }
  document.body.classList.toggle('modal-open', state.openLayers.length > 0);
  const origin = state.focusOrigins.get(overlay.id);
  if (restoreFocus && origin instanceof HTMLElement && document.contains(origin)) window.setTimeout(() => origin.focus(), 0);
  state.focusOrigins.delete(overlay.id);
}

function handleGlobalKeydown(event) {
  const id = state.openLayers.at(-1);
  const overlay = id ? document.getElementById(id) : null;
  if (!overlay) return;
  if (event.key === 'Escape') {
    event.preventDefault();
    closeLayer(overlay);
    return;
  }
  if (event.key !== 'Tab') return;
  const focusable = [...overlay.querySelectorAll('button:not(:disabled),input:not(:disabled),select:not(:disabled),textarea:not(:disabled),[tabindex]:not([tabindex="-1"])')]
    .filter((element) => !element.hidden && element.getClientRects().length > 0);
  if (!focusable.length) return;
  const first = focusable[0];
  const last = focusable.at(-1);
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault();
    last.focus();
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault();
    first.focus();
  }
}

function renderDrawer() {
  const entries = [
    ['▤', 'Servers', false, () => {}], ['＋', 'Add Server', false, openAddServer], ['◫', 'Runtime Support', false, showRuntimeSupport],
    ['◉', 'Device Status', false, showDeviceStatus], ['⚙', 'Settings', true, () => showNotice('Settings is planned and does not have a configuration screen yet.')],
    ['ⓘ', 'About Slopity', false, showAbout],
  ];
  elements.drawerNav.replaceChildren(...entries.map(([icon, label, planned, handler], index) => {
    const button = node('button', `drawer-link${index === 0 ? ' active' : ''}`);
    button.type = 'button';
    button.append(node('span', 'nav-icon', icon), node('span', '', label));
    if (planned) button.append(node('span', 'planned', 'Planned'));
    button.addEventListener('click', () => {
      if (!planned) closeLayer(elements.drawerOverlay);
      handler();
    });
    return button;
  }));
}

function showRuntimeSupport() {
  const content = document.createDocumentFragment();
  content.append(supportHeading('builtin', 'Runtime Support', 'Providers are marked ready only after real lifecycle proof.'));
  const list = node('div', 'actions-list');
  for (const runtime of state.runtimes) {
    const row = node('div', 'action-row');
    const copy = node('span');
    copy.append(node('strong', '', runtimeLabel(runtime.runtime)), node('small', '', runtime.reason));
    row.append(copy, statusPill({ key: runtime.available ? 'running' : 'unavailable', label: runtime.available ? 'Ready' : 'Unavailable' }));
    list.append(row);
  }
  if (!state.runtimes.length) list.append(node('div', 'support-copy', 'Runtime catalog is unavailable until the native Tauri bridge loads.'));
  content.append(list);
  elements.templateSupportContent.replaceChildren(content);
  setLayerOpen(elements.templateOverlay, true, elements.templateSheet);
}

function showDeviceStatus() {
  const snapshot = state.snapshot;
  const content = document.createDocumentFragment();
  content.append(supportHeading('builtin', 'Device Status', snapshot ? `${snapshot.platform} · ${snapshot.architecture}` : 'Native device data unavailable'));
  if (snapshot) {
    const grid = node('div', 'detail-grid');
    const stats = [
      ['Safe server budget', snapshot.resourcePlan?.safeServerBudgetMib ? `${snapshot.resourcePlan.safeServerBudgetMib} MiB` : 'Probe pending'],
      ['Stored profiles', String(state.profiles.length)], ['Active servers', String(state.servers.filter((server) => isActiveState(server.state)).length)],
      ['Foreground service', snapshot.hostService?.foregroundServiceAvailable ? 'Compiled' : 'Unavailable'],
    ];
    for (const [label, value] of stats) {
      const stat = node('div', 'detail-stat');
      stat.append(node('small', '', label), node('strong', '', value));
      grid.append(stat);
    }
    content.append(grid, node('div', 'support-copy', snapshot.hostService?.reason || 'No additional host capability details are available.'));
  } else content.append(node('div', 'support-copy', 'Run Slopity through the native Tauri shell to read device and host capability information.'));
  elements.templateSupportContent.replaceChildren(content);
  setLayerOpen(elements.templateOverlay, true, elements.templateSheet);
}

function showAbout() {
  const content = document.createDocumentFragment();
  content.append(supportHeading('builtin', 'About Slopity', 'Portable server hosting with explicit runtime boundaries.'));
  content.append(node('div', 'support-copy', 'Slopity uses a Rust core, Tauri 2 shell, durable profile storage, and a built-in HTTP server shared across Linux and Android. External runtimes remain unavailable until each provider is implemented and proven rather than merely represented by UI.'));
  elements.templateSupportContent.replaceChildren(content);
  setLayerOpen(elements.templateOverlay, true, elements.templateSheet);
}

for (const closeButton of document.querySelectorAll('[data-close]')) {
  closeButton.addEventListener('click', () => {
    const overlays = { drawer: elements.drawerOverlay, add: elements.addOverlay, details: elements.detailsOverlay, actions: elements.actionsOverlay, template: elements.templateOverlay, editor: elements.editorOverlay };
    if (overlays[closeButton.dataset.close]) closeLayer(overlays[closeButton.dataset.close]);
  });
}

elements.menuButton.addEventListener('click', openDrawer);
elements.refreshButton.addEventListener('click', () => loadDashboard({ announce: true }));
elements.addServerButton.addEventListener('click', openAddServer);
elements.runtime.addEventListener('change', updateRuntimeFields);
elements.form.addEventListener('submit', saveProfile);
document.addEventListener('keydown', handleGlobalKeydown);
renderTemplates();
renderDrawer();
loadDashboard();
window.setInterval(refreshServerStates, 2_000);
