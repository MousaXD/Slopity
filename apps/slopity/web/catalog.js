import { node, runtimeLabel } from './dom.js';

export function createCatalogController({
  elements,
  state,
  openEditor,
  openSheet,
  closeSheet,
  showNotice,
  nextAvailablePort,
}) {
  function selectTemplate(templateId) {
    closeSheet('add-sheet', { restoreFocus: false });
    switch (templateId) {
      case 'http':
        openEditor(null, {
          name: 'My HTTP server',
          runtime: 'built-in-http',
          port: nextAvailablePort(8_080),
          memoryMib: 128,
          enabled: true,
          note: 'Available now: this profile starts Slopity’s fixed built-in Rust HTTP page and /health endpoint.',
        });
        break;
      case 'website':
        openEditor(null, {
          name: 'My Website',
          runtime: 'built-in-http',
          port: nextAvailablePort(8_080),
          memoryMib: 128,
          enabled: true,
          note: 'Website currently maps to the built-in HTTP probe foundation. It serves Slopity’s fixed page, not your own static files or application yet.',
        });
        break;
      case 'minecraft':
        openUnsupportedTemplate({
          title: 'Minecraft runtime is planned',
          subtitle: 'No JVM server is installed or launched',
          paragraphs: [
            'Slopity does not currently download Paper, execute JAR files, or claim Minecraft hosting support.',
            'You can save a disabled Java placeholder for future setup. It cannot start until a verified JVM provider is implemented.',
          ],
          actionLabel: 'Create disabled placeholder',
          preset: {
            name: 'Minecraft Server',
            runtime: 'java',
            port: nextAvailablePort(25_565),
            memoryMib: 2_048,
            enabled: false,
            arguments: ['-jar', 'paper.jar', '--nogui'],
            note: 'Configuration placeholder only. No JAR is downloaded or executed and Start remains unavailable.',
          },
        });
        break;
      case 'node':
        openUnsupportedTemplate({
          title: 'Node.js runtime is planned',
          subtitle: 'No Node.js provider is installed',
          paragraphs: [
            'Saving a profile does not install Node.js, run npm, or execute an application.',
            'A disabled placeholder can record intended configuration without pretending the runtime works.',
          ],
          actionLabel: 'Create disabled placeholder',
          preset: {
            name: 'Node.js App',
            runtime: 'node-js',
            port: nextAvailablePort(3_000),
            memoryMib: 512,
            enabled: false,
            arguments: ['server.js'],
            note: 'Configuration placeholder only. No Node.js process will be launched.',
          },
        });
        break;
      case 'import':
        openUnsupportedTemplate({
          title: 'Safe import is not implemented',
          subtitle: 'Slopity will not pretend to import a VPS',
          paragraphs: [
            'There is currently no remote VPS connection, archive import, credential flow, or executable payload import.',
            'You may create a disabled custom placeholder to record a name, port, and memory target only.',
          ],
          actionLabel: 'Create import placeholder',
          preset: {
            name: 'Imported Server',
            runtime: 'custom',
            port: nextAvailablePort(8_081),
            memoryMib: 256,
            enabled: false,
            note: 'Placeholder only. No files, credentials, remote server, or executable content are imported.',
          },
        });
        break;
      case 'custom':
        openEditor(null, {
          name: 'Custom Server',
          runtime: 'custom',
          port: nextAvailablePort(8_081),
          memoryMib: 256,
          enabled: false,
          note: 'Custom profiles are editable configuration only. No verified custom runtime provider is exposed by the UI.',
        });
        break;
      default:
        showNotice('Unknown template selection.', 'error');
    }
  }

  function openRuntimeSupport() {
    const content = node('ul', { className: 'info-list' });
    (state.snapshot?.runtimes ?? []).forEach((runtime) => {
      content.append(node('li', {
        text: `${runtimeLabel(runtime.runtime)}: ${runtime.available ? 'available' : 'unavailable'}. ${runtime.reason}`,
      }));
    });
    openInfoSheet({
      title: 'Runtime Support',
      subtitle: 'Honest capability boundary',
      content,
    });
  }

  function openDeviceStatus() {
    const snapshot = state.snapshot;
    const host = snapshot?.hostService;
    const budget = snapshot?.resourcePlan?.safeServerBudgetMib ?? 0;
    const content = node('ul', { className: 'info-list' }, [
      node('li', { text: `Platform: ${snapshot?.platform ?? 'unknown'} · ${snapshot?.architecture ?? 'unknown'}` }),
      node('li', { text: `Safe server memory budget: ${budget > 0 ? `${budget} MiB` : 'device probe pending'}` }),
      node('li', { text: host?.reason ?? 'Host-service capability is unavailable.' }),
      node('li', { text: host?.durableHostingAvailable ? 'Durable hosting is reported available.' : 'Durable Android background hosting is not claimed yet.' }),
    ]);
    openInfoSheet({
      title: 'Device Status',
      subtitle: 'Platform and host-service envelope',
      content,
    });
  }

  function openInfoSheet({ title, subtitle, paragraphs = [], content = null, actions = [] }) {
    elements.infoTitle.textContent = title;
    elements.infoSubtitle.textContent = subtitle;
    const children = paragraphs.map((paragraph) => node('p', { text: paragraph }));
    if (content) {
      children.push(content);
    }
    elements.infoContent.replaceChildren(...children);
    elements.infoActions.replaceChildren(...actions.map((action) => {
      const button = node('button', {
        className: action.kind === 'primary' ? 'primary-button' : 'quiet-button',
        text: action.label,
        attrs: { type: 'button' },
      });
      button.addEventListener('click', action.onClick);
      return button;
    }));
    openSheet('info-sheet');
  }

  function openUnsupportedTemplate({ title, subtitle, paragraphs, actionLabel, preset }) {
    openInfoSheet({
      title,
      subtitle,
      paragraphs,
      actions: [
        {
          label: actionLabel,
          kind: 'primary',
          onClick: () => {
            closeSheet('info-sheet', { restoreFocus: false });
            openEditor(null, preset);
          },
        },
      ],
    });
  }

  return { selectTemplate, openRuntimeSupport, openDeviceStatus, openInfoSheet };
}
