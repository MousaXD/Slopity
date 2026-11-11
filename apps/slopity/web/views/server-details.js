import { el, issueList, replace, statusPill } from '../components/dom.js';
import { profileLifecycle, runtimeBadge, runtimeLabel, titleCase } from '../domain/runtime.js';
import { collectResourceWarnings } from '../domain/resources.js';

export function renderServerDetails(document, target, profile, snapshot, validationIssues = []) {
  const lifecycle = profileLifecycle(profile, snapshot);
  const badge = runtimeBadge(profile, snapshot);
  const server = lifecycle.server;
  const runtime = lifecycle.availability;

  const header = el(document, 'div', { className: 'details-header' },
    el(document, 'div', { className: 'runtime-mark large', text: runtimeLabel(profile.runtime).slice(0, 2).toUpperCase(), attributes: { 'aria-hidden': 'true' } }),
    el(document, 'div', { className: 'details-title' },
      el(document, 'h2', { text: profile.name }),
      el(document, 'p', { text: `${runtimeLabel(profile.runtime)} · ${profile.networkScope === 'lan' ? 'LAN exposed' : 'Loopback only'} · port ${profile.port}` }),
    ),
    statusPill(document, badge.key, badge.label),
  );

  const facts = el(document, 'div', { className: 'details-facts' },
    info(document, 'Configuration', profile.enabled ? 'Enabled' : 'Disabled'),
    info(document, 'Runtime adapter', runtime.available ? 'Verified' : 'Unavailable'),
    info(document, 'Desired state', titleCase(server?.desiredState ?? 'stopped')),
    info(document, 'Observed state', titleCase(server?.state ?? 'stopped')),
    info(document, 'Memory budget', `${profile.memoryMib} MiB`),
    info(document, 'Bind address', server?.bindAddress || 'Not bound'),
    info(document, 'Requests', String(server?.requestCount ?? 0)),
    info(document, 'Process ID', server?.processId ?? 'Not applicable'),
  );

  const body = [header];
  if (!runtime.available) {
    body.push(el(document, 'div', { className: 'callout warning', text: runtime.reason || 'No verified runtime adapter is registered.' }));
  }
  if (profile.networkScope === 'lan') {
    body.push(el(document, 'div', { className: 'callout warning', text: 'LAN exposure is enabled. Other devices on the same local network may be able to reach this server.' }));
  }
  body.push(facts);

  if (server?.lastError) body.push(section(document, 'Last failure', el(document, 'pre', { className: 'failure-box', text: server.lastError })));
  if (server?.exit) body.push(section(document, 'Last exit', el(document, 'div', { className: 'callout', text: `${titleCase(server.exit.reason)}: ${server.exit.message}` })));

  const urls = server?.urls?.length
    ? server.urls.map((url) => el(document, 'code', { className: 'url-chip', text: url }))
    : [el(document, 'p', { className: 'muted', text: runtime.available ? 'No URL is available while the server is stopped.' : 'No URL is available for an unsupported runtime.' })];
  body.push(section(document, 'URLs', el(document, 'div', { className: 'url-list' }, ...urls)));

  const logs = server?.logs?.length
    ? server.logs.slice(-30).map((entry) => el(document, 'div', { className: `log-row log-${entry.level}` },
        el(document, 'span', { className: 'log-level', text: entry.level }),
        el(document, 'span', { className: 'log-message', text: entry.message }),
      ))
    : [el(document, 'p', { className: 'muted', text: 'No runtime logs recorded yet.' })];
  body.push(section(document, 'Recent logs', el(document, 'div', { className: 'log-list' }, ...logs)));

  const resourceWarnings = collectResourceWarnings(snapshot);
  const issues = [...validationIssues, ...resourceWarnings.map((warning) => ({ ...warning, severity: 'warning' }))];
  body.push(section(document, 'Validation & resources', issues.length
    ? issueList(document, issues)
    : el(document, 'p', { className: 'muted', text: 'No current validation or resource warnings.' })));

  replace(target, ...body);
}

function info(document, label, value) {
  return el(document, 'div', { className: 'detail-info' },
    el(document, 'span', { text: label }),
    el(document, 'strong', { text: value }),
  );
}

function section(document, title, content) {
  return el(document, 'section', { className: 'details-section' },
    el(document, 'h3', { text: title }),
    content,
  );
}
