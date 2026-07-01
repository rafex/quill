import { search } from '../api/client.js';
import type { SearchResultResponse } from '../types.js';

function esc(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

function resultCard(r: SearchResultResponse): string {
  return `
    <article class="result-card">
      <span class="badge">${esc(r.type)}</span>
      <h3>${esc(r.title)}</h3>
      <p>${esc(r.snippet)}</p>
      <small>Score: ${r.score.toFixed(3)}</small>
    </article>`;
}

export class QuillSearch extends HTMLElement {
  connectedCallback(): void {
    this.innerHTML = `
      <section class="search-panel">
        <form class="search-form">
          <input type="search" name="q" placeholder="Buscar posts y comentarios…" autocomplete="off" />
          <button type="submit">Buscar</button>
        </form>
        <div class="search-results"></div>
      </section>`;
    this.querySelector('form')!.addEventListener('submit', (e) => void this._onSubmit(e));
  }

  private async _onSubmit(e: Event): Promise<void> {
    e.preventDefault();
    const q = (this.querySelector('input[name=q]') as HTMLInputElement).value.trim();
    if (!q) return;
    const out = this.querySelector('.search-results') as HTMLElement;
    out.innerHTML = '<p class="muted">Buscando…</p>';
    try {
      const results = await search(q);
      out.innerHTML = results.length === 0
        ? '<p class="muted">Sin resultados.</p>'
        : results.map(resultCard).join('');
    } catch (err) {
      out.innerHTML = `<p class="error">${esc((err as Error).message)}</p>`;
    }
  }
}

customElements.define('quill-search', QuillSearch);
