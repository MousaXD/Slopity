const focusOrigins = new WeakMap();

function frame(callback) {
  const view = globalThis.requestAnimationFrame;
  if (typeof view === 'function') view(callback);
  else globalThis.setTimeout(callback, 0);
}

export function showDialog(dialog, focusTarget = null) {
  if (!dialog.open) {
    focusOrigins.set(dialog, dialog.ownerDocument?.activeElement ?? null);
    dialog.showModal();
  }
  frame(() => (focusTarget || dialog.querySelector('button,input,select,textarea'))?.focus());
}

export function closeDialog(dialog) {
  if (!dialog.open) return;
  const origin = focusOrigins.get(dialog);
  focusOrigins.delete(dialog);
  dialog.close();
  if (origin?.isConnected && typeof origin.focus === 'function') frame(() => origin.focus());
}

export function createConfirmController(dialog) {
  const title = dialog.querySelector('#confirm-title');
  const copy = dialog.querySelector('#confirm-copy');
  const confirm = dialog.querySelector('#confirm-action');
  const cancel = dialog.querySelector('#confirm-cancel');
  let resolvePending = null;

  cancel.addEventListener('click', () => settle(false));
  confirm.addEventListener('click', () => settle(true));
  dialog.addEventListener('cancel', (event) => {
    event.preventDefault();
    settle(false);
  });
  dialog.addEventListener('close', () => {
    if (resolvePending) settle(false);
  });

  function settle(value) {
    const resolve = resolvePending;
    resolvePending = null;
    closeDialog(dialog);
    resolve?.(value);
  }

  return function ask({ heading, message, action = 'Confirm', danger = false }) {
    title.textContent = heading;
    copy.textContent = message;
    confirm.textContent = action;
    confirm.className = danger ? 'button danger' : 'button primary';
    const promise = new Promise((resolve) => {
      resolvePending = resolve;
    });
    showDialog(dialog, confirm);
    return promise;
  };
}
