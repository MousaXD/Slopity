const tauriInvoke = window.__TAURI_INTERNALS__?.invoke;

const elements = {
  platform: document.querySelector('#platform-pill'),
  hostTitle: document.querySelector('#host-title'),
  hostState: document.querySelector('#host-state'),
  hostReason: document.querySelector('#host-reason'),
  memoryBudget: document.querySelector('#memory-budget'),
  memoryNote: document.querySelector('#memory-note'),
  runtimeCount: document.querySelector('#runtime-count'),
  profiles: document.querySelector('#profiles'),
  refresh: document.querySelector('#refresh'),
};

async function loadDashboard() {
  try {
    if (!tauriInvoke) {
      throw new Error('The native Tauri bridge is not available in this browser preview.');
    }

    const snapshot = await tauriInvoke('dashboard_snapshot');
    render(snapshot);
  } catch (error) {
    elements.hostTitle.textContent = 'Native bridge unavailable';
    elements.hostState.textContent = 'Preview only';
    elements.hostState.className = 'status blocked';
    elements.hostReason.textContent = String(error);
    elements.profiles.innerHTML = '<article class="card error">Run this frontend through the Tauri shell to read Rust state.</article>';
  }
}

function render(snapshot) {
  elements.platform.textContent = `${snapshot.platform} · ${snapshot.architecture}`;
  elements.hostTitle.textContent = snapshot.hostService.durableHostingAvailable
    ? 'Desktop hosting boundary ready'
    : 'Hosting proof still required';
  elements.hostState.textContent = snapshot.hostService.durableHostingAvailable ? 'Foundation ready' : 'Not claimed';
  elements.hostState.className = snapshot.hostService.durableHostingAvailable ? 'status ready' : 'status blocked';
  elements.hostReason.textContent = snapshot.hostService.reason;

  const budget = snapshot.resourcePlan.safeServerBudgetMib;
  elements.memoryBudget.textContent = budget > 0 ? `${budget} MiB` : 'Probe pending';
  elements.memoryNote.textContent = snapshot.resourcePlan.warning
    ?? 'The current shell does not yet include a platform memory probe.';

  const ready = snapshot.runtimes.filter((runtime) => runtime.available).length;
  elements.runtimeCount.textContent = `${ready} ready / ${snapshot.runtimes.length} known`;

  elements.profiles.replaceChildren(...snapshot.samples.map(profileCard));
}

function profileCard(profile) {
  const card = document.createElement('article');
  card.className = 'card profile-card';
  const runtime = profile.runtime.replaceAll('-', ' ');
  card.innerHTML = `
    <p class="eyebrow">${escapeHtml(runtime)}</p>
    <h3>${escapeHtml(profile.name)}</h3>
    <p class="muted">Disabled sample. A runtime provider must be installed and verified before launch.</p>
    <div class="profile-meta">
      <span>Port ${profile.port}</span>
      <span>${profile.memoryMib} MiB</span>
      <span>${escapeHtml(profile.networkScope)}</span>
    </div>
  `;
  return card;
}

function escapeHtml(value) {
  return String(value)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#039;');
}

elements.refresh.addEventListener('click', loadDashboard);
loadDashboard();
