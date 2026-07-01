import type {
  UserResponse,
  CategoryResponse,
  TopicResponse,
  PostResponse,
  CommentResponse,
  SearchResultResponse,
} from '../types.js';

const USERS_BASE   = 'http://localhost:8080';
const CONTENT_BASE = 'http://localhost:8081';
const SEARCH_BASE  = 'http://localhost:8082';

interface RequestOptions extends RequestInit {
  headers?: Record<string, string>;
}

async function req<T>(base: string, path: string, options: RequestOptions = {}): Promise<T> {
  const res = await fetch(`${base}${path}`, {
    headers: { 'Content-Type': 'application/json', ...options.headers },
    ...options,
  });
  const body = await res.json() as { error?: string } & T;
  if (!res.ok) {
    throw Object.assign(new Error(body.error ?? res.statusText), { status: res.status });
  }
  return body;
}

// Users
export const createUser = (username: string, email: string): Promise<UserResponse> =>
  req(USERS_BASE, '/users', { method: 'POST', body: JSON.stringify({ username, email }) });

export const getUser = (id: string): Promise<UserResponse> =>
  req(USERS_BASE, `/users/${id}`);

// Content
export const createCategory = (name: string, slug: string): Promise<CategoryResponse> =>
  req(CONTENT_BASE, '/categories', { method: 'POST', body: JSON.stringify({ name, slug }) });

export const createTopic = (category_id: string, title: string, slug: string): Promise<TopicResponse> =>
  req(CONTENT_BASE, '/topics', { method: 'POST', body: JSON.stringify({ category_id, title, slug }) });

export const createPost = (topic_id: string, title: string, slug: string, body: string): Promise<PostResponse> =>
  req(CONTENT_BASE, '/posts', { method: 'POST', body: JSON.stringify({ topic_id, title, slug, body }) });

export const createComment = (post_id: string, body: string): Promise<CommentResponse> =>
  req(CONTENT_BASE, '/comments', { method: 'POST', body: JSON.stringify({ post_id, body }) });

export const getCategory = (id: string): Promise<CategoryResponse> =>
  req(CONTENT_BASE, `/categories/${id}`);

export const getTopic = (id: string): Promise<TopicResponse> =>
  req(CONTENT_BASE, `/topics/${id}`);

export const getPost = (id: string): Promise<PostResponse> =>
  req(CONTENT_BASE, `/posts/${id}`);

export const getComment = (id: string): Promise<CommentResponse> =>
  req(CONTENT_BASE, `/comments/${id}`);

// Search
export const search = (q: string, limit = 10): Promise<SearchResultResponse[]> =>
  req(SEARCH_BASE, `/search?q=${encodeURIComponent(q)}&limit=${limit}`);
