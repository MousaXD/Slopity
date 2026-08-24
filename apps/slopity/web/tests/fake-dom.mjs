class FakeNode {
  constructor(tagName = '#text', text = '') {
    this.tagName = tagName.toUpperCase();
    this.children = [];
    this.attributes = new Map();
    this.listeners = new Map();
    this.className = '';
    this.type = '';
    this.disabled = false;
    this.hidden = false;
    this.value = '';
    this.checked = false;
    this._text = String(text);
  }

  set textContent(value) {
    this._text = String(value ?? '');
    this.children = [];
  }

  get textContent() {
    return `${this._text}${this.children.map((child) => child.textContent ?? '').join('')}`;
  }

  append(...children) {
    for (const child of children) {
      if (child === null || child === undefined) continue;
      this.children.push(typeof child === 'string' ? new FakeNode('#text', child) : child);
    }
  }

  replaceChildren(...children) {
    this.children = [];
    this._text = '';
    this.append(...children);
  }

  setAttribute(name, value) {
    this.attributes.set(name, String(value));
  }

  getAttribute(name) {
    return this.attributes.get(name) ?? null;
  }

  addEventListener(name, handler) {
    this.listeners.set(name, handler);
  }

  dispatch(name) {
    return this.listeners.get(name)?.({ preventDefault() {}, stopPropagation() {} });
  }
}

export class FakeDocument {
  createElement(tagName) {
    return new FakeNode(tagName);
  }

  createTextNode(text) {
    return new FakeNode('#text', text);
  }
}

export function findByTag(node, tagName) {
  const target = tagName.toUpperCase();
  const found = [];
  if (node.tagName === target) found.push(node);
  for (const child of node.children ?? []) found.push(...findByTag(child, tagName));
  return found;
}

export function findByClass(node, className) {
  const found = [];
  if ((node.className ?? '').split(/\s+/).includes(className)) found.push(node);
  for (const child of node.children ?? []) found.push(...findByClass(child, className));
  return found;
}
