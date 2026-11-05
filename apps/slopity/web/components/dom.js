export function el(document, tag, options = {}, ...children) {
  const node = document.createElement(tag);
  if (options.className) node.className = options.className;
  if (options.text !== undefined && options.text !== null) node.textContent = String(options.text);
  if (options.type) node.type = options.type;
  if (options.value !== undefined) node.value = String(options.value);
  if (options.name) node.name = options.name;
  if (options.disabled !== undefined) node.disabled = Boolean(options.disabled);
  if (options.hidden !== undefined) node.hidden = Boolean(options.hidden);
  if (options.checked !== undefined) node.checked = Boolean(options.checked);
  if (options.attributes) {
    for (const [name, value] of Object.entries(options.attributes)) {
      if (value !== null && value !== undefined) node.setAttribute(name, String(value));
    }
  }
  if (options.on) {
    for (const [eventName, handler] of Object.entries(options.on)) node.addEventListener(eventName, handler);
  }
  for (const child of children.flat()) {
    if (child === null || child === undefined || child === false) continue;
    if (typeof child === 'string' || typeof child === 'number') node.append(document.createTextNode(String(child)));
    else node.append(child);
  }
  return node;
}

export function replace(node, ...children) {
  node.replaceChildren(...children.flat().filter((child) => child !== null && child !== undefined));
}

export function setText(node, value, fallback = 'Unavailable') {
  node.textContent = value === null || value === undefined || value === '' ? fallback : String(value);
}

export function statusPill(document, key, label) {
  return el(document, 'span', { className: `status-pill status-${key}`, text: label });
}

export function issueList(document, issues = [], className = 'issue-list') {
  const list = el(document, 'div', { className, attributes: { role: 'list' } });
  for (const issue of issues) {
    list.append(
      el(document, 'div', {
        className: `issue issue-${issue.severity || 'warning'}`,
        text: issue.message,
        attributes: { role: 'listitem' },
      }),
    );
  }
  return list;
}
