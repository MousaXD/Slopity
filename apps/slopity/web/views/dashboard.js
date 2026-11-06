import { el, issueList, replace, statusPill } from '../components/dom.js';
import { renderServerCard } from '../components/server-card.js';
import { collectResourceWarnings, dashboardMetrics, recoveryMessages } from '../domain/resources.js';
import { availabilityFor, runtimeLabel } from '../domain/runtime.js';
import { selectProfiles } from '../domain/profile-query.js';

// Save the action identity, not a DOM node: cards are replaced during live polling.
export function captureServerActionFocus(document, target) {
  const active = document.activeElement;
  if (!active || typeof target.contains !== 'function' || !target.contains(active)) return () => {};
  const profileId = active.closest?.('[data-profile-id]')?.getAttribute('data-profile-id');
  const action = active.getAttribute?.('data-server-action');
  if (!profileId || !['lifecycle', 'details'].includes(action)) return () => {};
  return () => {
    for (const card of target.querySelectorAll('[data-profile-id]')) {
      if (card.getAttribute('data-profile-id') !== profileId) continue;
      const next = card.querySelector(`[data-server-action="${action}"]`);
      const fallback = card.querySelector('[data-server-action="details"]');
      const targetAction = next && !next.disabled ? next : fallback;
      if (typeof targetAction?.focus === 'function') targetAction.focus({ preventScroll: true });
      break;
    }
  };
}

export function renderDashboard(document, elements, snapshot, actions = {}, filters = {}) {
  if (!snapshot) return;
  renderMetrics(document, elements.metrics, snapshot);
  renderRuntimeAvailability(document, elements.runtimeGrid, snapshot);
  renderWarnings(document, elements.resourceWarnings, snapshot);
  renderRecovery(document, elements.recoveryNotices, snapshot);
  renderServers(document, elements.serverGrid, snapshot, actions, filters, elements.resultCount);
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

function renderServers(document, target, snapshot, actions, filters = {}, resultCount = null) {
  const restoreFocus = captureServerActionFocus(document, target);
  const allProfiles = snapshot.profiles ?? [];
  const profiles = selectProfiles(snapshot, filters);
  if (resultCount) resultCount.textContent = `Showing ${profiles.length} of ${allProfiles.length} saved profiles`;
  if (!profiles.length) {
    const empty = !allProfiles.length;
    replace(target, el(document, 'div', { className: 'empty-state' },
      el(document, 'strong', { text: empty ? 'No saved servers' : 'No matching profiles' }),
      el(document, 'p', { text: empty
        ? 'Create a profile. Built-in HTTP can run now; other runtimes remain configuration-only until a verified adapter exists.'
        : 'Try another search, runtime, or status filter. No profiles were changed.' }),
    ));
    return;
  }
  replace(target, ...profiles.map((profile) => renderServerCard(document, profile, snapshot, actions)));
  restoreFocus();
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
