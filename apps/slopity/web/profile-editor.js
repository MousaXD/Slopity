import { previewValidation } from './preview.js';
import { renderValidation } from './views.js';

export function createProfileEditor({
  elements,
  state,
  overlays,
  previewMode,
  tauriInvoke,
  showNotice,
  nextAvailablePort,
  nextProfileId,
  runProfileCommand,
}) {
  async function loadProfileValidation(profile, target) {
    if (!target) {
      return;
    }
    if (previewMode) {
      renderValidation(target, previewValidation(profile));
      return;
    }
    try {
      renderValidation(target, await tauriInvoke('validate_server_profile', { profile }));
    } catch (error) {
      renderValidation(target, [{ severity: 'error', message: String(error) }]);
    }
  }

  function openEditor(profile = null, preset = {}) {
    state.editingId = profile?.id ?? null;
    elements.editorTitle.textContent = profile ? `Edit ${profile.name}` : 'Create profile';
    elements.editorSubtitle.textContent = profile
      ? 'Update saved configuration'
      : 'Configure a saved server profile';
    elements.id.readOnly = Boolean(profile);
    elements.id.value = profile?.id ?? nextProfileId();
    elements.name.value = profile?.name ?? preset.name ?? 'My HTTP server';
    elements.runtime.value = profile?.runtime ?? preset.runtime ?? 'built-in-http';
    elements.network.value = profile?.networkScope ?? preset.networkScope ?? 'loopback';
    elements.port.value = String(profile?.port ?? preset.port ?? nextAvailablePort(8_080));
    elements.memory.value = String(profile?.memoryMib ?? preset.memoryMib ?? 128);
    elements.executable.value = profile?.executable ?? preset.executable ?? '';
    elements.directory.value = profile?.workingDirectory ?? preset.workingDirectory ?? '';
    elements.arguments.value = (profile?.arguments ?? preset.arguments ?? []).join('\n');
    elements.enabled.checked = profile?.enabled ?? preset.enabled ?? true;
    elements.templateNote.hidden = !preset.note;
    elements.templateNote.textContent = preset.note ?? '';
    elements.editorValidation.replaceChildren();
    updateRuntimeFields();
    overlays.openSheet('editor-sheet');
    scheduleEditorValidation();
  }

  function closeEditor() {
    state.editingId = null;
    window.clearTimeout(state.validationTimer);
    elements.form.reset();
    elements.editorValidation.replaceChildren();
    overlays.closeSheet('editor-sheet');
  }

  function updateRuntimeFields() {
    const builtIn = elements.runtime.value === 'built-in-http';
    [elements.executable, elements.directory, elements.arguments].forEach((field) => {
      field.disabled = builtIn;
    });
    elements.enabled.disabled = false;
    scheduleEditorValidation();
  }

  function readProfileForm() {
    const builtIn = elements.runtime.value === 'built-in-http';
    return {
      id: elements.id.value.trim(),
      name: elements.name.value.trim(),
      runtime: elements.runtime.value,
      executable: builtIn ? null : nullable(elements.executable.value),
      arguments: builtIn
        ? []
        : elements.arguments.value.split('\n').map((value) => value.trim()).filter(Boolean),
      workingDirectory: builtIn ? null : nullable(elements.directory.value),
      port: Number(elements.port.value),
      memoryMib: Number(elements.memory.value),
      networkScope: elements.network.value,
      enabled: builtIn ? elements.enabled.checked : false,
    };
  }

  function scheduleEditorValidation() {
    window.clearTimeout(state.validationTimer);
    state.validationTimer = window.setTimeout(async () => {
      if (!elements.editorSheet.hidden) {
        await loadProfileValidation(readProfileForm(), elements.editorValidation);
      }
    }, 180);
  }

  async function saveProfile(event) {
    event.preventDefault();
    if (previewMode) {
      showNotice('Profile writes are disabled in browser preview mode.', 'error');
      return;
    }
    const profile = readProfileForm();
    const command = state.editingId ? 'update_server_profile' : 'create_server_profile';
    if (await runProfileCommand(command, { profile }, `${profile.name} saved.`)) {
      closeEditor();
    }
  }

  return {
    closeEditor,
    loadProfileValidation,
    openEditor,
    saveProfile,
    scheduleEditorValidation,
    updateRuntimeFields,
  };
}

function nullable(value) {
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}
