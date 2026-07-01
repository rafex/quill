import type { QuillEvent } from '../types.js';

const CONTENT_BASE = 'http://localhost:8081';

type EventHandler = (event: QuillEvent) => void;

let _es: EventSource | null = null;
const _listeners = new Map<symbol, EventHandler>();

export function subscribeEvents(handler: EventHandler): () => void {
  if (!_es) {
    _es = new EventSource(`${CONTENT_BASE}/events`);
    _es.onmessage = (e: MessageEvent<string>) => {
      try {
        const event = JSON.parse(e.data) as QuillEvent;
        _listeners.forEach((h) => h(event));
      } catch (_) {
        // malformed event — ignore
      }
    };
    _es.onerror = () => {
      _es?.close();
      _es = null;
      if (_listeners.size > 0) {
        setTimeout(() => subscribeEvents(() => undefined), 3000);
      }
    };
  }
  const id = Symbol();
  _listeners.set(id, handler);
  return () => { _listeners.delete(id); };
}
