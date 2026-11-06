export function formatMib(value) {
  return Number.isFinite(value) ? `${Number(value).toLocaleString()} MiB` : 'Unavailable';
}

export function formatPercent(value) {
  return Number.isFinite(value) ? `${value}%` : 'Unavailable';
}

export function dashboardMetrics(snapshot = {}) {
  const profiles = snapshot.profiles ?? [];
  const servers = snapshot.servers ?? [];
  const telemetry = snapshot.deviceTelemetry ?? {};
  const active = servers.filter((server) => ['starting', 'running', 'stopping'].includes(server.state)).length;
  const battery = Number.isFinite(telemetry.batteryPercentage) ? `${telemetry.batteryPercentage}%` : 'Unavailable';
  const batteryHint = typeof telemetry.charging === 'boolean' ? (telemetry.charging ? 'Charging' : 'On battery') : 'Charging state unavailable';
  const thermal = telemetry.thermalStatus || 'Unavailable';
  const thermalHint = Number.isFinite(telemetry.batteryTemperatureCelsius)
    ? `${telemetry.batteryTemperatureCelsius.toFixed(1)} °C battery`
    : 'Temperature unavailable';
  return [
    { label: 'Saved servers', value: String(profiles.length), hint: `${profiles.filter((profile) => profile.enabled).length} enabled` },
    { label: 'Active servers', value: String(active), hint: `${snapshot.resourceAccounting?.activeOrReservedServerCount ?? 0} active or reserved` },
    { label: 'Available RAM', value: formatMib(snapshot.capability?.availableMemoryMib), hint: `Total ${formatMib(snapshot.capability?.totalMemoryMib)}` },
    { label: 'Safe server budget', value: formatMib(snapshot.resourcePlan?.safeServerBudgetMib), hint: `${formatMib(snapshot.resourceAccounting?.remainingSafeBudgetMib)} remaining · ${formatMib(snapshot.resourcePlan?.hostReserveMib)} host reserve` },
    { label: 'Reserved RAM', value: formatMib(snapshot.resourceAccounting?.reservedMemoryMib), hint: `${snapshot.resourcePlan?.recommendedConcurrentServers ?? 0} recommended concurrent` },
    { label: 'CPU', value: snapshot.capability?.logicalCpus > 0 ? `${snapshot.capability.logicalCpus} logical` : 'Unavailable', hint: 'Backend-reported host capacity' },
    { label: 'Battery', value: battery, hint: batteryHint },
    { label: 'Thermal', value: thermal, hint: thermalHint },
  ];
}

export function collectResourceWarnings(snapshot = {}) {
  const warnings = [...(snapshot.resourcePlan?.warnings ?? []), ...(snapshot.resourceAccounting?.warnings ?? [])];
  return [...new Map(warnings.map((warning) => [`${warning.code}:${warning.message}`, warning])).values()];
}

export function deviceTelemetryRows(snapshot = {}) {
  const telemetry = snapshot.deviceTelemetry ?? {};
  return [
    ['Telemetry source', telemetry.source || 'Unavailable'],
    ['Available RAM', formatMib(snapshot.capability?.availableMemoryMib)],
    ['Total RAM', formatMib(snapshot.capability?.totalMemoryMib)],
    ['Battery', formatPercent(telemetry.batteryPercentage)],
    ['Charging', typeof telemetry.charging === 'boolean' ? (telemetry.charging ? 'Yes' : 'No') : 'Unavailable'],
    ['Battery temperature', Number.isFinite(telemetry.batteryTemperatureCelsius) ? `${telemetry.batteryTemperatureCelsius.toFixed(1)} °C` : 'Unavailable'],
    ['Thermal status', telemetry.thermalStatus || 'Unavailable'],
    ['Free app storage', formatMib(telemetry.freeStorageMib)],
  ];
}

export function recoveryMessages(snapshot = {}) {
  return (snapshot.profileRecoveryNotices ?? []).map((notice) => notice.message || notice.reason || String(notice));
}
