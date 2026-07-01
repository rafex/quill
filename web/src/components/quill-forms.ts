import { loadWasm } from '../wasm/loader.js';
import {
  createUser,
  createCategory,
  createTopic,
  createPost,
  createComment,
} from '../api/client.js';

function esc(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

function setMsg(el: Element, text: string, kind: 'ok' | 'error'): void {
  const p = el.querySelector('.msg') as HTMLElement;
  p.textContent = text;
  p.className = `msg ${kind}`;
}

function input(el: Element, name: string): string {
  return (el.querySelector(`[name=${name}]`) as HTMLInputElement | HTMLTextAreaElement).value.trim();
}

function wireSlugAuto(form: Element, sourceName: string, slugName: string): void {
  const slugEl = form.querySelector(`[name=${slugName}]`) as HTMLInputElement;
  form.querySelector(`[name=${sourceName}]`)!.addEventListener('input', async () => {
    if (!slugEl.dataset['manual']) {
      const wasm = await loadWasm();
      slugEl.value = wasm.generate_slug((form.querySelector(`[name=${sourceName}]`) as HTMLInputElement).value);
    }
  });
  slugEl.addEventListener('input', () => { slugEl.dataset['manual'] = '1'; });
}

// ── Register User ──────────────────────────────────────────────────────────────
export class QuillRegisterForm extends HTMLElement {
  connectedCallback(): void {
    this.innerHTML = `
      <form class="card">
        <h2>Registrar usuario</h2>
        <label>Username <input name="username" required /></label>
        <label>Email    <input name="email" type="email" required /></label>
        <button type="submit">Crear</button>
        <p class="msg"></p>
      </form>`;
    this.querySelector('form')!.addEventListener('submit', (e) => void this._submit(e));
  }

  private async _submit(e: Event): Promise<void> {
    e.preventDefault();
    const wasm = await loadWasm();
    const email = input(this, 'email');
    if (!wasm.validate_email(email)) { setMsg(this, 'Email inválido', 'error'); return; }
    try {
      const user = await createUser(input(this, 'username'), email);
      setMsg(this, `✔ Usuario creado: ${esc(user.id)}`, 'ok');
    } catch (err) { setMsg(this, (err as Error).message, 'error'); }
  }
}

// ── Create Category ────────────────────────────────────────────────────────────
export class QuillCategoryForm extends HTMLElement {
  connectedCallback(): void {
    this.innerHTML = `
      <form class="card">
        <h2>Nueva categoría</h2>
        <label>Nombre <input name="name" required /></label>
        <label>Slug   <input name="slug" placeholder="auto-generado" /></label>
        <button type="submit">Crear</button>
        <p class="msg"></p>
      </form>`;
    wireSlugAuto(this, 'name', 'slug');
    this.querySelector('form')!.addEventListener('submit', (e) => void this._submit(e));
  }

  private async _submit(e: Event): Promise<void> {
    e.preventDefault();
    const wasm = await loadWasm();
    const slug = input(this, 'slug');
    if (!wasm.validate_slug(slug)) { setMsg(this, 'Slug inválido (a-z, 0-9, guiones)', 'error'); return; }
    try {
      const cat = await createCategory(input(this, 'name'), slug);
      setMsg(this, `✔ Categoría: ${esc(cat.id)}`, 'ok');
      this.dispatchEvent(new CustomEvent('quill:category-created', { detail: cat, bubbles: true }));
    } catch (err) { setMsg(this, (err as Error).message, 'error'); }
  }
}

// ── Create Topic ───────────────────────────────────────────────────────────────
export class QuillTopicForm extends HTMLElement {
  connectedCallback(): void {
    this.innerHTML = `
      <form class="card">
        <h2>Nuevo tema</h2>
        <label>Category ID <input name="category_id" required /></label>
        <label>Título      <input name="title" required /></label>
        <label>Slug        <input name="slug" placeholder="auto-generado" /></label>
        <button type="submit">Crear</button>
        <p class="msg"></p>
      </form>`;
    wireSlugAuto(this, 'title', 'slug');
    this.querySelector('form')!.addEventListener('submit', (e) => void this._submit(e));
  }

  private async _submit(e: Event): Promise<void> {
    e.preventDefault();
    const wasm = await loadWasm();
    const slug = input(this, 'slug');
    if (!wasm.validate_slug(slug)) { setMsg(this, 'Slug inválido', 'error'); return; }
    try {
      const topic = await createTopic(input(this, 'category_id'), input(this, 'title'), slug);
      setMsg(this, `✔ Tema: ${esc(topic.id)}`, 'ok');
      this.dispatchEvent(new CustomEvent('quill:topic-created', { detail: topic, bubbles: true }));
    } catch (err) { setMsg(this, (err as Error).message, 'error'); }
  }
}

// ── Create Post ────────────────────────────────────────────────────────────────
export class QuillPostForm extends HTMLElement {
  connectedCallback(): void {
    this.innerHTML = `
      <form class="card">
        <h2>Nuevo post</h2>
        <label>Topic ID <input name="topic_id" required /></label>
        <label>Título   <input name="title" required /></label>
        <label>Slug     <input name="slug" placeholder="auto-generado" /></label>
        <label>Cuerpo   <textarea name="body" rows="5" required></textarea></label>
        <button type="submit">Publicar</button>
        <p class="msg"></p>
      </form>`;
    wireSlugAuto(this, 'title', 'slug');
    this.querySelector('form')!.addEventListener('submit', (e) => void this._submit(e));
  }

  private async _submit(e: Event): Promise<void> {
    e.preventDefault();
    const wasm = await loadWasm();
    const slug = input(this, 'slug');
    if (!wasm.validate_slug(slug)) { setMsg(this, 'Slug inválido', 'error'); return; }
    try {
      const post = await createPost(input(this, 'topic_id'), input(this, 'title'), slug, input(this, 'body'));
      setMsg(this, `✔ Post creado: ${esc(post.id)}`, 'ok');
      this.dispatchEvent(new CustomEvent('quill:post-created', { detail: post, bubbles: true }));
    } catch (err) { setMsg(this, (err as Error).message, 'error'); }
  }
}

// ── Create Comment ─────────────────────────────────────────────────────────────
export class QuillCommentForm extends HTMLElement {
  connectedCallback(): void {
    this.innerHTML = `
      <form class="card">
        <h2>Comentar</h2>
        <label>Post ID <input name="post_id" required /></label>
        <label>Texto   <textarea name="body" rows="3" required></textarea></label>
        <button type="submit">Comentar</button>
        <p class="msg"></p>
      </form>`;
    this.querySelector('form')!.addEventListener('submit', (e) => void this._submit(e));
  }

  private async _submit(e: Event): Promise<void> {
    e.preventDefault();
    try {
      const comment = await createComment(input(this, 'post_id'), input(this, 'body'));
      setMsg(this, `✔ Comentario: ${esc(comment.id)}`, 'ok');
      this.dispatchEvent(new CustomEvent('quill:comment-created', { detail: comment, bubbles: true }));
    } catch (err) { setMsg(this, (err as Error).message, 'error'); }
  }
}

customElements.define('quill-register-form', QuillRegisterForm);
customElements.define('quill-category-form', QuillCategoryForm);
customElements.define('quill-topic-form', QuillTopicForm);
customElements.define('quill-post-form', QuillPostForm);
customElements.define('quill-comment-form', QuillCommentForm);
