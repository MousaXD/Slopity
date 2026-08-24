import { deviceTelemetryRows } from '../domain/resources.js';
import { el, replace } from '../components/dom.js';

export function renderDeviceStatus(document, target, snapshot) {
  const rows = deviceTelemetryRows(snapshot);
  replace(target, ...rows.map(([label, value]) =>
    el(document, 'div', { className: 'device-row' },
      el(document, 'span', { text: label }),
      el(document, 'strong', { text: value }),
    ),
  ));
}
