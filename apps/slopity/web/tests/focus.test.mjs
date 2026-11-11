import test from 'node:test';
import assert from 'node:assert/strict';
import { captureServerActionFocus } from '../views/dashboard.js';

function scenario({ within = true, action = 'lifecycle', nextDisabled = false, exists = true } = {}) {
  let focused = null;
  const oldAction = {
    getAttribute(name) { return name === 'data-server-action' ? action : null; },
    closest() { return { getAttribute() { return 'http-1'; } }; },
  };
  const nextAction = { disabled: nextDisabled, focus() { focused = 'lifecycle'; } };
  const details = { disabled: false, focus() { focused = 'details'; } };
  const card = {
    getAttribute() { return 'http-1'; },
    querySelector(selector) { return selector.includes('lifecycle') ? nextAction : details; },
  };
  const target = {
    contains(node) { return within && node === oldAction; },
    querySelectorAll() { return exists ? [card] : []; },
  };
  return {
    restore: captureServerActionFocus({ activeElement: oldAction }, target),
    focused: () => focused,
  };
}

test('live redraw returns keyboard focus to the same lifecycle action', () => {
  const state = scenario();
  state.restore();
  assert.equal(state.focused(), 'lifecycle');
});

test('disabled lifecycle action falls back to accessible Details button', () => {
  const state = scenario({ nextDisabled: true });
  state.restore();
  assert.equal(state.focused(), 'details');
});

test('polling does not steal focus from an unrelated view or missing profile', () => {
  const otherView = scenario({ within: false });
  otherView.restore();
  assert.equal(otherView.focused(), null);
  const removed = scenario({ exists: false });
  removed.restore();
  assert.equal(removed.focused(), null);
});
