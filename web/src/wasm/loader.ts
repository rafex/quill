import init, * as QuillWasm from '../../wasm/pkg/quill_wasm.js';

export type { QuillWasm };

let _loaded: typeof QuillWasm | null = null;

export async function loadWasm(): Promise<typeof QuillWasm> {
  if (_loaded) return _loaded;
  await init();
  _loaded = QuillWasm;
  return _loaded;
}
