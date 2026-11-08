import { el, replace } from '../components/dom.js';
import { runtimeLabel } from '../domain/runtime.js';

export function renderImportPreview(document, target, preview) {
  if (!preview) {
    replace(target);
    return;
  }
  const { entries, importable, skipped } = preview;
  replace(target,
    el(document, 'p', { className: 'muted', text:
      `${importable} ready to import · ${skipped} conflicts skipped. All imported profiles are disabled; no server will start.` }),
    el(document, 'ul', { className: 'import-preview-list' }, ...entries.map(({ profile, reason }) =>
      el(document, 'li', { className: reason ? 'import-conflict' : 'import-ready' },
        el(document, 'strong', { text: profile.name }),
        el(document, 'span', { text: `${runtimeLabel(profile.runtime)} · ${profile.id} · port ${profile.port}` }),
        el(document, 'small', { text: reason || 'Ready · imported disabled' }),
      ),
    )),
  );
}
