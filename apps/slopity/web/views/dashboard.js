import { el, issueList, replace, statusPill } from '../components/dom.js';
import { renderServerCard } from '../components/server-card.js';
import { collectResourceWarnings, dashboardMetrics, recoveryMessages } from '../domain/resources.js';
import { availabilityFor, runtimeLabel } from '../domain/runtime.js';

export function renderDashboard(document, elements, snapshot, actions = {}) {
  if (!snapshot) return;
  renderMetrics(document, elements.metrics, snapshot);
  renderRuntimeAvailability(document, elements.runtimeGrid, snapshot);
  renderWarnings(document, elements.resourceWarnings, snapshot);
  renderRecovery(document, elements.recoveryNotices, snapshot);
  renderServers(document, elements.serverGrid, snapshot, actions);
  renderHostStatus(document, elements.hostStatus, snapshot);
}

function renderMetrics(document, target, snapshot) {
  replace(target, ...dashboardMetrics(snapshot).map((metric) =>
    el(document, 'article', { className: 'metric-card' },
      el(document, 'span', { className: 'metric-label', text: metric.label }),
      el(document, 'strong', { className: 'metric-value', text: metric.value }),
      el(document, 'span', { className: 'metric-hint', text: metric.hint }),
    ),
  ));
}

function renderRuntimeAvailability(document, target, snapshot) {
  const ordered = ['built-in-http', 'java', 'node-js', 'python', 'php', 'native', 'custom'];
  replace(target, ...ordered.map((runtime) => {
    const availability = availabilityFor(runtime, snapshot.runtimes ?? []);
    return el(document, 'article', { className: 'runtime-card' },
      el(document, 'div', { className: 'runtime-card-copy' },
        el(document, 'strong', { text: runtimeLabel(runtime) }),
        el(document, 'small', { text: availability.available ? 'Verified adapter registered' : availability.reason }),
      ),
      statusPill(document, availability.available ? 'running' : 'unavailable', availability.available ? 'Ready' : 'Unavailable'),
    );
  }));
}

function renderWarnings(document, target, snapshot) {
  const warnings = collectResourceWarnings(snapshot);
  target.hidden = warnings.length === 0;
  replace(target, warnings.length ? issueList(document, warnings.map((warning) => ({ ...warning, severity: 'warning' }))) : null);
}

function renderRecovery(document, target, snapshot) {
  const messages = recoveryMessages(snapshot);
  target.hidden = messages.length === 0;
  replace(target, ...messages.map((message) => el(document, 'div', { className: 'recovery-notice', text: message })));
}

function renderServers(document, target, snapshot, actions) {
  const profiles = snapshot.profiles ?? [];
  if (!profiles.length) {
    replace(target, el(document, 'div', { className: 'empty-state' },
      el(document, 'strong', { text: 'No saved servers' }),
      el(document, 'p', { text: 'Create a profile. Built-in HTTP can run now; other runtimes remain configuration-only until a verified adapter exists.' }),
    ));
    return;
  }
  replace(target, ...profiles.map((profile) => renderServerCard(document, profile, snapshot, actions)));
}

function renderHostStatus(document, target, snapshot) {
  const host = snapshot.hostServiceStatus ?? {};
  const capability = snapshot.hostService ?? {};
  const label = host.active
    ? `${host.activeServerCount ?? 0} hosted`
    : capability.durableHostingAvailable
      ? 'Host ready'
      : 'Host capability limited';
  target.className = `host-chip ${host.active ? 'active' : ''}`;
  target.textContent = label;
}
