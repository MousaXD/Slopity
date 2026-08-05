import {
  createServerIcon,
  createStatusPill,
  detailItem,
  isActiveState,
  node,
  profileTypeLabel,
  runtimeLabel,
  statusFor,
  templates,
} from './dom.js';

export function renderBridgeError(elements, error) {
  elements.platformLabel.textContent = 'Native bridge unavailable';
  elements.profileSummary.textContent = 'Unable to read persisted profiles';
  elements.profiles.replaceChildren(node('article', { className: 'error-state' }, [
    node('h2', { text: 'Open Slopity through Tauri' }),
    node('p', { text: String(error) }),
  ]));
  elements.profiles.setAttribute('aria-busy', 'false');
}

export function renderProfiles({ elements, profiles, servers, schemaVersion, busy, onAdd, onDetails, onMenu }) {
  const enabled = profiles.filter((profile) => profile.enabled).length;
  const active = servers.filter((server) => isActiveState(server.state)).length;
  elements.profileSummary.textContent = `${profiles.length} saved · ${enabled} enabled · ${active} active · schema v${schemaVersion ?? '?'}`;

  if (profiles.length === 0) {
    const add = node('button', {
      className: 'primary-button',
      text: 'Add your first server',
      attrs: { type: 'button' },
    });
    add.addEventListener('click', onAdd);
    elements.profiles.replaceChildren(node('article', { className: 'empty-state' }, [
      node('h2', { text: 'No saved servers yet' }),
      node('p', { text: 'Create a built-in HTTP profile or save an honest placeholder for a future runtime.' }),
      add,
    ]));
  } else {
    const cards = profiles.map((profile) => profileCard(profile, servers, onDetails, onMenu));
    elements.profiles.replaceChildren(...cards);
  }

  elements.profiles.setAttribute('aria-busy', String(busy));
}

export function renderTemplates(elements, onSelect) {
  const options = templates.map((template) => {
    const button = node('button', {
      className: 'template-option',
      attrs: { type: 'button', role: 'listitem' },
    }, [
      createServerIcon(template),
      node('span', { className: 'template-copy' }, [
        node('strong', { text: template.title }),
        node('small', { text: template.description }),
        node('em', { text: template.badge }),
      ]),
      node('span', { className: 'button-chevron', text: '›', attrs: { 'aria-hidden': 'true' } }),
    ]);
    button.addEventListener('click', () => onSelect(template.id));
    return button;
  });
  elements.templateList.replaceChildren(...options);
}

export function renderDetails({ elements, profile, server, busy, previewMode }) {
  const status = statusFor(profile, server);
  const active = isActiveState(server?.state);
  const builtIn = profile.runtime === 'built-in-http';

  elements.detailsIcon.replaceWith(createDetailsIcon(profile));
  elements.detailsIcon = document.querySelector('#details-icon');
  elements.detailsTitle.textContent = profile.name;
  elements.detailsSubtitle.textContent = `${profileTypeLabel(profile)} · ${status.label}`;

  const statePanel = node('section', { className: 'detail-panel' }, [
    node('h3', { text: 'Runtime state' }),
    node('div', { className: 'detail-state-row' }, [
      node('strong', { text: profileTypeLabel(profile) }),
      createStatusPill(status),
    ]),
  ]);

  if (!builtIn) {
    statePanel.append(node('p', {
      className: 'runtime-warning',
      text: 'This profile has no verified runtime provider. It cannot start from Slopity yet.',
    }));
  } else if (!profile.enabled) {
    statePanel.append(node('p', {
      className: 'runtime-warning',
      text: 'Enable this profile before starting the built-in HTTP server.',
    }));
  }
  if (server?.lastError) {
    statePanel.append(node('div', { className: 'validation-item error', text: server.lastError }));
  }

  const configuration = node('section', { className: 'detail-panel' }, [
    node('h3', { text: 'Profile configuration' }),
    node('div', { className: 'detail-grid' }, [
      detailItem('Profile ID', profile.id),
      detailItem('Runtime', runtimeLabel(profile.runtime)),
      detailItem('Port', String(profile.port)),
      detailItem('Memory', `${profile.memoryMib} MiB`),
      detailItem('Network', profile.networkScope),
      detailItem('Configuration', profile.enabled ? 'Enabled' : 'Disabled'),
    ]),
  ]);

  const runtimePanel = node('section', { className: 'detail-panel' }, [
    node('h3', { text: 'Live server' }),
    node('div', { className: 'detail-grid' }, [
      detailItem('Requests', String(server?.requestCount ?? 0)),
      detailItem('Bind address', server?.bindAddress ?? 'Not bound'),
    ]),
  ]);

  const validationSlot = node('div', {
    className: 'validation-list',
    attrs: { 'data-validation-slot': '' },
  }, [node('p', { className: 'empty-copy', text: 'Checking profile validation…' })]);

  elements.detailsContent.replaceChildren(
    statePanel,
    configuration,
    node('section', { className: 'detail-panel' }, [
      node('h3', { text: 'Available URLs' }),
      createUrlList(server?.urls ?? []),
    ]),
    runtimePanel,
    node('section', { className: 'detail-panel' }, [
      node('h3', { text: 'Recent runtime logs' }),
      createLogList(server?.logs ?? []),
    ]),
    node('section', { className: 'detail-panel' }, [
      node('h3', { text: 'Validation' }),
      validationSlot,
    ]),
  );

  const controls = [];
  if (builtIn) {
    controls.push(actionButton(active ? 'Stop server' : 'Start server', active ? 'stop' : 'start', {
      kind: active ? 'danger' : 'primary',
      disabled: !active && !profile.enabled,
      busy,
      previewMode,
    }));
  }
  controls.push(actionButton('Edit', 'edit', { disabled: active, busy, previewMode }));
  controls.push(actionButton('Clone', 'clone', { busy, previewMode }));
  controls.push(actionButton(profile.enabled ? 'Disable' : 'Enable', 'toggle', { disabled: active, busy, previewMode }));
  controls.push(actionButton('Delete', 'delete', { kind: 'danger', disabled: active, busy, previewMode }));
  elements.detailsActions.replaceChildren(...controls);
  return validationSlot;
}

export function renderValidation(target, issues) {
  if (!target) {
    return;
  }
  if (!issues || issues.length === 0) {
    target.replaceChildren(node('p', { className: 'empty-copy', text: 'No validation issues.' }));
    return;
  }
  target.replaceChildren(...issues.map((issue) => node('div', {
    className: `validation-item ${issue.severity ?? 'warning'}`,
    text: issue.message,
  })));
}

function profileCard(profile, servers, onDetails, onMenu) {
  const server = servers.find((candidate) => candidate.serverId === profile.id) ?? null;
  const status = statusFor(profile, server);
  const card = node('article', { className: `server-card is-${status.key}` });
  const open = node('button', {
    className: 'server-card-main',
    attrs: { type: 'button', 'aria-label': `Open details for ${profile.name}` },
  }, [
    createServerIcon(profile),
    node('div', { className: 'server-card-copy' }, [
      node('h2', { text: profile.name }),
      node('p', { text: profileTypeLabel(profile) }),
      node('div', { className: 'server-card-meta' }, [createStatusPill(status)]),
    ]),
  ]);
  open.addEventListener('click', () => onDetails(profile.id));

  const menuButton = node('button', {
    className: 'card-menu-button',
    attrs: {
      type: 'button',
      'aria-label': `Actions for ${profile.name}`,
      'aria-haspopup': 'menu',
      'aria-expanded': 'false',
    },
  }, [node('span', { className: 'kebab', attrs: { 'aria-hidden': 'true' } }, [
    node('i'), node('i'), node('i'),
  ])]);
  menuButton.addEventListener('click', (event) => {
    event.stopPropagation();
    onMenu(profile, menuButton);
  });
  card.append(open, menuButton);
  return card;
}

function createDetailsIcon(profile) {
  const icon = createServerIcon(profile);
  icon.id = 'details-icon';
  return icon;
}

function createUrlList(urls) {
  if (urls.length === 0) {
    return node('p', { className: 'empty-copy', text: 'Start the built-in HTTP server to see reachable URLs.' });
  }
  const list = node('div', { className: 'url-list' });
  urls.forEach((url) => {
    const copy = node('button', {
      className: 'copy-button',
      text: 'Copy',
      attrs: { type: 'button', 'aria-label': `Copy ${url}`, 'data-copy-url': url },
    });
    list.append(node('div', { className: 'url-row' }, [node('code', { text: url }), copy]));
  });
  return list;
}

function createLogList(logs) {
  if (logs.length === 0) {
    return node('p', { className: 'empty-copy', text: 'No runtime logs are available yet.' });
  }
  const list = node('div', { className: 'log-list' });
  logs.slice(-20).reverse().forEach((entry) => {
    list.append(node('div', { className: `log-entry log-${entry.level}` }, [
      node('span', { className: 'log-level', text: entry.level }),
      node('span', { text: entry.message }),
    ]));
  });
  return list;
}

function actionButton(label, action, { kind = 'quiet', disabled = false, busy = false, previewMode = false } = {}) {
  const classNames = {
    primary: 'primary-button',
    danger: 'danger-button',
    quiet: 'quiet-button',
  };
  const button = node('button', {
    className: classNames[kind] ?? classNames.quiet,
    text: label,
    attrs: { type: 'button' },
  });
  button.dataset.detailAction = action;
  button.dataset.mutation = 'true';
  button.dataset.locked = String(disabled);
  button.disabled = disabled || busy || previewMode;
  return button;
}
