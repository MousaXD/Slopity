const tauriInvoke = window.__TAURI_INTERNALS__?.invoke;

function stat(label, value) {
  const item = document.createElement('div');
  item.className = 'detail-stat';
  const caption = document.createElement('small');
  caption.textContent = label;
  const data = document.createElement('strong');
  data.textContent = value;
  item.append(caption, data);
  return item;
}

function formatMib(value) {
  return Number.isFinite(value) ? `${value} MiB` : 'Unavailable';
}

function optionalAndroidStats(telemetry) {
  const stats = [];
  if (Number.isFinite(telemetry?.batteryPercentage)) {
    stats.push(stat('Battery', `${telemetry.batteryPercentage}%`));
  }
  if (typeof telemetry?.charging === 'boolean') {
    stats.push(stat('Charging', telemetry.charging ? 'Yes' : 'No'));
  }
  if (Number.isFinite(telemetry?.batteryTemperatureCelsius)) {
    stats.push(stat('Battery temperature', `${telemetry.batteryTemperatureCelsius.toFixed(1)} °C`));
  }
  if (telemetry?.thermalStatus) {
    stats.push(stat('Thermal status', telemetry.thermalStatus));
  }
  if (Number.isFinite(telemetry?.freeStorageMib)) {
    stats.push(stat('Free app storage', `${telemetry.freeStorageMib} MiB`));
  }
  return stats;
}

function renderResourceStatus(snapshot) {
  const content = document.querySelector('#template-support-content');
  const grid = content?.querySelector('.detail-grid');
  if (!content || !grid) return;

  const capability = snapshot.capability ?? {};
  const accounting = snapshot.resourceAccounting ?? {};
  const plan = snapshot.resourcePlan ?? {};
  const telemetry = snapshot.deviceTelemetry ?? {};
  const stats = [
    stat('Total RAM', formatMib(capability.totalMemoryMib)),
    stat('Available RAM', formatMib(capability.availableMemoryMib)),
    stat('Safe server budget', formatMib(plan.safeServerBudgetMib)),
    stat('Reserved server memory', formatMib(accounting.reservedMemoryMib ?? 0)),
    stat('Remaining safe budget', formatMib(accounting.remainingSafeBudgetMib)),
    stat('Logical CPUs', capability.logicalCpus > 0 ? String(capability.logicalCpus) : 'Unavailable'),
    stat('Active / reserved servers', String(accounting.activeOrReservedServerCount ?? 0)),
  ];
  stats.push(...optionalAndroidStats(telemetry));
  grid.replaceChildren(...stats);

  content.querySelectorAll('.resource-accounting-details').forEach((element) => element.remove());
  const details = document.createElement('div');
  details.className = 'resource-accounting-details';

  const source = document.createElement('div');
  source.className = 'support-copy';
  source.textContent = telemetry.source
    ? `Telemetry source: ${telemetry.source}. Values that the platform cannot provide remain unavailable.`
    : 'Device telemetry source is unavailable.';
  details.append(source);

  const warnings = [...(plan.warnings ?? []), ...(accounting.warnings ?? [])];
  const uniqueWarnings = [...new Map(warnings.map((warning) => [`${warning.code}:${warning.message}`, warning])).values()];
  if (uniqueWarnings.length) {
    const list = document.createElement('div');
    list.className = 'validation-list';
    for (const warning of uniqueWarnings) {
      const item = document.createElement('div');
      item.className = 'validation-item warning';
      item.textContent = warning.message;
      list.append(item);
    }
    details.append(list);
  }
  content.append(details);
}

async function refreshDeviceStatus() {
  if (!tauriInvoke) return;
  try {
    renderResourceStatus(await tauriInvoke('dashboard_snapshot'));
  } catch (error) {
    const content = document.querySelector('#template-support-content');
    if (!content) return;
    const message = document.createElement('div');
    message.className = 'validation-item warning resource-accounting-details';
    message.textContent = `Resource telemetry refresh failed: ${String(error)}`;
    content.append(message);
  }
}

window.addEventListener('DOMContentLoaded', () => {
  const button = [...document.querySelectorAll('#drawer-nav button')]
    .find((candidate) => candidate.textContent.includes('Device Status'));
  button?.addEventListener('click', refreshDeviceStatus);
});
