export const templates = [
  {
    id: 'minecraft',
    title: 'Minecraft Server',
    description: 'Prepare a disabled profile for future JVM support',
    badge: 'Runtime support coming later',
    icon: 'MC',
    iconClass: 'icon-minecraft',
  },
  {
    id: 'website',
    title: 'Website',
    description: 'Create the current built-in web probe foundation',
    badge: 'Built-in HTTP foundation',
    icon: '◎',
    iconClass: 'icon-built-in-http',
  },
  {
    id: 'import',
    title: 'Import Server',
    description: 'Review the planned safe profile import flow',
    badge: 'Import not implemented',
    icon: '⇧',
    iconClass: 'icon-import',
  },
  {
    id: 'node',
    title: 'Node.js App',
    description: 'Prepare a disabled profile for future Node.js support',
    badge: 'Runtime support coming later',
    icon: 'JS',
    iconClass: 'icon-node-js',
  },
  {
    id: 'http',
    title: 'Built-in HTTP Server',
    description: 'Create a working loopback-first Rust HTTP probe',
    badge: 'Available now',
    icon: '◎',
    iconClass: 'icon-built-in-http',
  },
  {
    id: 'custom',
    title: 'Custom Template',
    description: 'Open a blank editable profile without claiming runtime support',
    badge: 'Configuration only',
    icon: '>_',
    iconClass: 'icon-custom',
  },
];

export function node(tagName, options = {}, children = []) {
  const element = document.createElement(tagName);
  if (options.className) {
    element.className = options.className;
  }
  if (options.text !== undefined) {
    element.textContent = String(options.text);
  }
  if (options.attrs) {
    Object.entries(options.attrs).forEach(([name, value]) => {
      if (value !== null && value !== undefined) {
        element.setAttribute(name, String(value));
      }
    });
  }
  children.filter(Boolean).forEach((child) => element.append(child));
  return element;
}

export function createServerIcon(profileOrTemplate) {
  const runtime = profileOrTemplate.runtime ?? profileOrTemplate.iconClass?.replace('icon-', '') ?? 'custom';
  const iconClass = profileOrTemplate.iconClass ?? iconClassForRuntime(runtime, profileOrTemplate.name);
  const iconText = profileOrTemplate.icon ?? iconTextForRuntime(runtime, profileOrTemplate.name);
  return node('div', {
    className: `server-icon ${iconClass}`,
    attrs: { 'aria-hidden': 'true' },
  }, [node('span', { text: iconText })]);
}

export function profileTypeLabel(profile) {
  const labels = {
    'built-in-http': 'Built-in HTTP Server',
    java: profile.name.toLowerCase().includes('minecraft') ? 'Minecraft Server' : 'Java Profile',
    'node-js': 'Node.js App',
    python: 'Python App',
    php: 'PHP App',
    native: 'Native Profile',
    custom: 'Custom Template',
  };
  return labels[profile.runtime] ?? 'Saved Server';
}

export function runtimeLabel(runtime) {
  const labels = {
    'built-in-http': 'Built-in HTTP',
    java: 'Java',
    'node-js': 'Node.js',
    python: 'Python',
    php: 'PHP',
    native: 'Native',
    custom: 'Custom',
  };
  return labels[runtime] ?? String(runtime).replaceAll('-', ' ');
}

export function statusFor(profile, server) {
  if (profile.runtime !== 'built-in-http') {
    return { key: 'unavailable', label: 'Unavailable' };
  }
  if (server?.state) {
    return { key: server.state, label: server.state };
  }
  if (!profile.enabled) {
    return { key: 'disabled', label: 'Disabled' };
  }
  return { key: 'stopped', label: 'Stopped' };
}

export function createStatusPill(status) {
  return node('span', {
    className: `status-pill status-${status.key}`,
    text: status.label,
  });
}

export function detailItem(label, value) {
  return node('div', { className: 'detail-item' }, [
    node('span', { text: label }),
    node('strong', { text: value }),
  ]);
}

export function isActiveState(serverState) {
  return ['starting', 'running', 'stopping'].includes(serverState);
}

function iconClassForRuntime(runtime, name = '') {
  if (runtime === 'java' && name.toLowerCase().includes('minecraft')) {
    return 'icon-minecraft';
  }
  const classes = {
    'built-in-http': 'icon-built-in-http',
    java: 'icon-java',
    'node-js': 'icon-node-js',
    python: 'icon-python',
    php: 'icon-php',
    native: 'icon-native',
    custom: 'icon-custom',
  };
  return classes[runtime] ?? 'icon-custom';
}

function iconTextForRuntime(runtime, name = '') {
  if (runtime === 'java' && name.toLowerCase().includes('minecraft')) {
    return 'MC';
  }
  const icons = {
    'built-in-http': '◎',
    java: 'J',
    'node-js': 'JS',
    python: 'Py',
    php: 'PHP',
    native: '▣',
    custom: '>_',
  };
  return icons[runtime] ?? '▣';
}
