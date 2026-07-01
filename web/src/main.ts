import './components/quill-search.js';
import './components/quill-forms.js';

type RouteKey = '' | 'search' | 'new';
type Renderer = (outlet: HTMLElement) => void;

const ROUTES: Record<RouteKey, Renderer> = {
  '':       renderHome,
  'search': renderSearch,
  'new':    renderNew,
};

function getRoute(): RouteKey {
  const fragment = location.hash.replace(/^#\/?/, '').split('/')[0];
  return (fragment in ROUTES ? fragment : '') as RouteKey;
}

function renderHome(outlet: HTMLElement): void {
  outlet.innerHTML = `
    <h1>Quill Forum</h1>
    <p class="muted">Foro ligero construido con Rust + WebAssembly + TypeScript.</p>
    <nav class="home-nav">
      <a href="#search">🔍 Buscar</a>
      <a href="#new">✏️ Publicar</a>
    </nav>`;
}

function renderSearch(outlet: HTMLElement): void {
  outlet.innerHTML = '<quill-search></quill-search>';
}

function renderNew(outlet: HTMLElement): void {
  outlet.innerHTML = `
    <h1>Publicar</h1>
    <div class="forms-grid">
      <quill-register-form></quill-register-form>
      <quill-category-form></quill-category-form>
      <quill-topic-form></quill-topic-form>
      <quill-post-form></quill-post-form>
      <quill-comment-form></quill-comment-form>
    </div>`;
}

function route(): void {
  const key = getRoute();
  const outlet = document.getElementById('outlet') as HTMLElement;
  ROUTES[key](outlet);

  document.querySelectorAll<HTMLAnchorElement>('nav.top-nav a').forEach((a) => {
    const href = a.getAttribute('href')?.replace('#', '') ?? '';
    a.classList.toggle('active', href === key);
  });
}

window.addEventListener('hashchange', route);
document.addEventListener('DOMContentLoaded', route);
