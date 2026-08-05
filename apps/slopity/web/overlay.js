export function createOverlayController({ elements, state, closeCardMenu }) {
  function openDrawer() {
    closeCardMenu();
    state.drawerReturnFocus = document.activeElement;
    elements.drawer.hidden = false;
    elements.drawerBackdrop.hidden = false;
    void elements.drawer.offsetWidth;
    elements.drawer.classList.add('is-open');
    elements.drawerBackdrop.classList.add('is-visible');
    elements.drawer.setAttribute('aria-hidden', 'false');
    elements.openDrawer.setAttribute('aria-expanded', 'true');
    syncBodyLock();
    elements.drawer.querySelector('button')?.focus();
  }

  function closeDrawer({ restoreFocus = true } = {}) {
    if (elements.drawer.getAttribute('aria-hidden') === 'true') {
      return;
    }
    elements.drawer.classList.remove('is-open');
    elements.drawerBackdrop.classList.remove('is-visible');
    elements.drawer.setAttribute('aria-hidden', 'true');
    elements.openDrawer.setAttribute('aria-expanded', 'false');
    window.setTimeout(() => {
      elements.drawerBackdrop.hidden = true;
      syncBodyLock();
    }, 210);
    if (restoreFocus) {
      state.drawerReturnFocus?.focus?.();
    }
  }

  function openSheet(sheetId, returnFocus = document.activeElement) {
    closeCardMenu();
    if (state.currentSheet && state.currentSheet !== sheetId) {
      closeSheet(state.currentSheet, { restoreFocus: false, immediate: true });
    }
    const sheet = document.querySelector(`#${sheetId}`);
    if (!sheet) {
      return;
    }
    state.currentSheet = sheetId;
    state.sheetReturnFocus = returnFocus;
    sheet.hidden = false;
    elements.modalBackdrop.hidden = false;
    void sheet.offsetWidth;
    sheet.classList.add('is-open');
    elements.modalBackdrop.classList.add('is-visible');
    syncBodyLock();
    window.setTimeout(() => firstFocusable(sheet)?.focus(), 30);
  }

  function closeSheet(sheetId = state.currentSheet, { restoreFocus = true, immediate = false } = {}) {
    if (!sheetId) {
      return;
    }
    const sheet = document.querySelector(`#${sheetId}`);
    if (!sheet || sheet.hidden) {
      return;
    }
    sheet.classList.remove('is-open');
    elements.modalBackdrop.classList.remove('is-visible');
    if (state.currentSheet === sheetId) {
      state.currentSheet = null;
    }
    const finish = () => {
      sheet.hidden = true;
      if (!state.currentSheet) {
        elements.modalBackdrop.hidden = true;
      }
      syncBodyLock();
    };
    if (immediate) {
      finish();
    } else {
      window.setTimeout(finish, 230);
    }
    if (restoreFocus) {
      state.sheetReturnFocus?.focus?.();
    }
  }

  function handleKeydown(event) {
    if (event.key === 'Escape') {
      if (!elements.cardMenu.hidden) {
        closeCardMenu();
        return;
      }
      if (state.currentSheet) {
        closeSheet(state.currentSheet);
        return;
      }
      closeDrawer();
      return;
    }

    if (state.currentSheet) {
      const sheet = document.querySelector(`#${state.currentSheet}`);
      if (sheet) {
        trapFocus(event, sheet);
      }
    } else if (elements.drawer.classList.contains('is-open')) {
      trapFocus(event, elements.drawer);
    }
  }

  function syncBodyLock() {
    const locked = elements.drawer.classList.contains('is-open') || Boolean(state.currentSheet);
    document.body.classList.toggle('is-locked', locked);
  }

  return { openDrawer, closeDrawer, openSheet, closeSheet, handleKeydown };
}

function firstFocusable(container) {
  return container.querySelector('button:not(:disabled), input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex="-1"])');
}

function trapFocus(event, container) {
  if (event.key !== 'Tab') {
    return;
  }
  const focusable = [...container.querySelectorAll('button:not(:disabled), input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex="-1"])')]
    .filter((element) => !element.hidden && element.offsetParent !== null);
  if (focusable.length === 0) {
    event.preventDefault();
    return;
  }
  const first = focusable[0];
  const last = focusable.at(-1);
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault();
    last.focus();
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault();
    first.focus();
  }
}
