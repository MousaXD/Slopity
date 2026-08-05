export const previewSnapshot = {
  application: 'Slopity',
  platform: 'preview',
  architecture: 'responsive-ui',
  hostService: {
    platform: 'preview',
    foregroundServiceAvailable: false,
    durableHostingAvailable: false,
    reason: 'Visual preview only. Native hosting is not connected.',
  },
  runtimes: [
    { runtime: 'built-in-http', available: true, reason: 'The harmless built-in Rust HTTP probe is compiled into Slopity.' },
    { runtime: 'java', available: false, reason: 'No verified external runtime provider is installed.' },
    { runtime: 'node-js', available: false, reason: 'No verified external runtime provider is installed.' },
    { runtime: 'python', available: false, reason: 'No verified external runtime provider is installed.' },
    { runtime: 'php', available: false, reason: 'No verified external runtime provider is installed.' },
    { runtime: 'native', available: false, reason: 'No verified external runtime provider is installed.' },
    { runtime: 'custom', available: false, reason: 'No verified external runtime provider is installed.' },
  ],
  profiles: [
    profile('http-example', 'Built-in HTTP', 'built-in-http', 8_080, 128, true),
    profile('website-example', 'My Website', 'built-in-http', 8_081, 128, true),
    profile('paper-example', 'Minecraft Placeholder', 'java', 25_565, 2_048, false, ['-jar', 'paper.jar', '--nogui']),
    profile('node-example', 'Node API Placeholder', 'node-js', 3_000, 512, false, ['server.js']),
    profile('custom-example', 'Custom Template', 'custom', 8_082, 256, false),
  ],
  servers: [
    {
      serverId: 'http-example',
      state: 'running',
      bindAddress: '127.0.0.1:8080',
      urls: ['http://127.0.0.1:8080'],
      requestCount: 12,
      logs: [
        { sequence: 1, level: 'info', message: 'Built-in HTTP server listening on 127.0.0.1:8080' },
        { sequence: 2, level: 'info', message: 'Served GET /health' },
      ],
      lastError: null,
    },
    {
      serverId: 'website-example',
      state: 'stopped',
      bindAddress: '127.0.0.1:8081',
      urls: [],
      requestCount: 0,
      logs: [],
      lastError: null,
    },
  ],
  profileSchemaVersion: 1,
  resourcePlan: {
    safeServerBudgetMib: 0,
    warning: 'Device memory probe pending.',
  },
};

export function previewValidation(profileValue) {
  const issues = [];
  if (!profileValue.id.trim()) {
    issues.push({ severity: 'error', message: 'Profile ID cannot be empty.' });
  }
  if (!profileValue.name.trim()) {
    issues.push({ severity: 'error', message: 'Profile name cannot be empty.' });
  }
  if (profileValue.runtime !== 'built-in-http') {
    issues.push({ severity: 'warning', message: 'No executable/runtime provider is configured; this profile is not runnable yet.' });
  }
  if (profileValue.networkScope === 'lan') {
    issues.push({ severity: 'warning', message: 'LAN exposure allows other devices on the local network to connect.' });
  }
  return issues;
}

export async function previewInvoke(command) {
  if (command === 'dashboard_snapshot') {
    return structuredClone(previewSnapshot);
  }
  if (command === 'list_builtin_http_servers') {
    return structuredClone(previewSnapshot.servers);
  }
  if (command === 'validate_server_profile') {
    return [];
  }
  throw new Error('This action is unavailable in visual browser preview mode.');
}

function profile(id, name, runtime, port, memoryMib, enabled, args = []) {
  return {
    id,
    name,
    runtime,
    executable: null,
    arguments: args,
    workingDirectory: null,
    port,
    memoryMib,
    networkScope: 'loopback',
    enabled,
  };
}
