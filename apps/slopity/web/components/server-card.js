import { el, statusPill } from './dom.js';
import { profileLifecycle, runtimeBadge, runtimeLabel, titleCase } from '../domain/runtime.js';

export function renderServerCard(document, profile, snapshot, actions = {}) {
  const lifecycle = profileLifecycle(profile, snapshot);
  const badge = runtimeBadge(profile, snapshot);
  const card = el(document, 'article', {
    className: 'server-card',
    attributes: { 'data-profile-id': profile.id, 'aria-label': `${profile.name} server profile` },
  });

  const heading = el(document, 'div', { className: 'server-card-heading' },
    el(document, 'div', { className: 'runtime-mark', text: runtimeLabel(profile.runtime).slice(0, 2).toUpperCase(), attributes: { 'aria-hidden': 'true' } }),
    el(document, 'div', { className: 'server-card-title' },
      el(document, 'h3', { text: profile.name }),
      el(document, 'p', { text: `${runtimeLabel(profile.runtime)} · ${profile.networkScope === 'lan' ? 'LAN' : 'Loopback'} · :${profile.port}` }),
    ),
    statusPill(document, badge.key, badge.label),
  );

  const facts = el(document, 'dl', { className: 'server-facts' },
    fact(document, 'Configuration', profile.enabled ? 'Enabled' : 'Disabled'),
    fact(document, 'Memory', `${profile.memoryMib} MiB`),
    fact(document, 'Observed', titleCase(lifecycle.state)),
    fact(document, 'Desired', titleCase(lifecycle.server?.desiredState ?? 'stopped')),
  );

  const footer = el(document, 'div', { className: 'server-card-actions' });
  const lifecycleButton = el(document, 'button', {
    className: lifecycle.active ? 'button danger' : 'button primary',
    type: 'button',
    text: lifecycle.active ? 'Stop' : lifecycle.availability.available ? 'Start' : 'Unavailable',
    disabled: lifecycle.active ? false : !lifecycle.runnable,
    attributes: {
      title: lifecycle.active ? 'Stop this server' : lifecycle.startDisabledReason,
      'aria-label': lifecycle.active ? `Stop ${profile.name}` : `Start ${profile.name}`,
    },
    on: { click: () => (lifecycle.active ? actions.onStop?.(profile) : actions.onStart?.(profile)) },
  });
  const details = el(document, 'button', {
    className: 'button secondary',
    type: 'button',
    text: 'Details',
    on: { click: () => actions.onDetails?.(profile) },
  });
  footer.append(lifecycleButton, details);

  if (!lifecycle.availability.available) {
    card.append(
      heading,
      el(document, 'p', { className: 'runtime-warning', text: lifecycle.availability.reason || 'This runtime is not registered.' }),
      facts,
      footer,
    );
  } else {
    card.append(heading, facts, footer);
  }
  return card;
}

function fact(document, label, value) {
  return el(document, 'div', { className: 'fact' },
    el(document, 'dt', { text: label }),
    el(document, 'dd', { text: value }),
  );
}
