import { el, replace } from '../components/dom.js';
import { draftToProfile, emptyDraft, profileToDraft } from '../domain/profile.js';
import { runtimeLabel } from '../domain/runtime.js';

const FIELD_IDS = {
  id: 'profile-id',
  name: 'profile-name',
  runtime: 'profile-runtime',
  networkScope: 'profile-network',
  port: 'profile-port',
  memoryMib: 'profile-memory',
  executable: 'profile-executable',
  workingDirectory: 'profile-directory',
  arguments: 'profile-arguments',
  enabled: 'profile-enabled',
};

export function createProfileEditor(document, dialog) {
  const form = dialog.querySelector('#profile-form');
  const title = dialog.querySelector('#editor-title');
  const note = dialog.querySelector('#editor-note');
  const unsupported = dialog.querySelector('#runtime-editor-warning');
  const externalFields = dialog.querySelector('#external-runtime-fields');
  const fieldErrors = new Map();
  for (const [field, id] of Object.entries(FIELD_IDS)) {
    const input = dialog.querySelector(`#${id}`);
    if (input) fieldErrors.set(field, dialog.querySelector(`[data-error-for="${field}"]`));
  }

  function input(field) {
    return dialog.querySelector(`#${FIELD_IDS[field]}`);
  }

  function setDraft(draft, { editing = false, runtimeAvailability = null } = {}) {
    title.textContent = editing ? `Edit ${draft.name}` : 'Create server profile';
    note.textContent = editing
      ? 'Update saved configuration. Active servers must be stopped before editing.'
      : 'Profiles describe configuration. Only verified runtime adapters can be started.';
    input('id').readOnly = editing;
    for (const field of Object.keys(FIELD_IDS)) {
      const control = input(field);
      if (!control) continue;
      if (control.type === 'checkbox') control.checked = Boolean(draft[field]);
      else control.value = draft[field] ?? '';
    }
    clearIssues();
    updateRuntimeState(runtimeAvailability);
  }

  function readDraft() {
    const draft = {};
    for (const field of Object.keys(FIELD_IDS)) {
      const control = input(field);
      draft[field] = control.type === 'checkbox' ? control.checked : control.value;
    }
    return draft;
  }

  function updateRuntimeState(runtimeAvailability = null) {
    const runtime = input('runtime').value;
    const builtIn = runtime === 'built-in-http';
    externalFields.hidden = builtIn;
    for (const field of ['executable', 'workingDirectory', 'arguments']) input(field).disabled = builtIn;
    unsupported.hidden = builtIn || runtimeAvailability?.available;
    unsupported.textContent = builtIn
      ? ''
      : runtimeAvailability?.reason || `${runtimeLabel(runtime)} is configuration-only until a verified adapter is registered.`;
    if (!builtIn && !input('id').readOnly) input('enabled').checked = false;
  }

  function clearIssues() {
    for (const [field, node] of fieldErrors) {
      const control = input(field);
      if (node) {
        node.hidden = true;
        node.textContent = '';
      }
      control?.removeAttribute('aria-invalid');
    }
    const summary = dialog.querySelector('#form-validation');
    summary.hidden = true;
    replace(summary);
  }

  function renderIssues(issues = []) {
    clearIssues();
    const summary = dialog.querySelector('#form-validation');
    const unresolved = [];
    for (const issue of issues) {
      const target = issue.field ? fieldErrors.get(issue.field) : null;
      const control = issue.field ? input(issue.field) : null;
      if (target) {
        target.hidden = false;
        target.textContent = issue.message;
        if (issue.severity === 'error') control?.setAttribute('aria-invalid', 'true');
      } else {
        unresolved.push(issue);
      }
    }
    const errors = issues.filter((issue) => issue.severity === 'error');
    if (unresolved.length || errors.length) {
      summary.hidden = false;
      replace(summary, ...issues.map((issue) =>
        el(document, 'div', { className: `issue issue-${issue.severity || 'warning'}`, text: issue.message }),
      ));
    }
  }

  function focusFirstError(issues = []) {
    const first = issues.find((issue) => issue.severity === 'error' && issue.field && input(issue.field));
    input(first?.field)?.focus();
  }

  input('runtime').addEventListener('change', () => updateRuntimeState());

  return {
    form,
    setDraft,
    readDraft,
    readProfile: () => draftToProfile(readDraft()),
    renderIssues,
    focusFirstError,
    updateRuntimeState,
    emptyDraft,
    profileToDraft,
  };
}
