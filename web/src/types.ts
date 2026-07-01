export interface UserResponse {
  id: string;
  username: string;
  email: string;
  created_at: string;
}

export interface CategoryResponse {
  id: string;
  name: string;
  slug: string;
  created_at: string;
}

export interface TopicResponse {
  id: string;
  category_id: string;
  title: string;
  slug: string;
  created_at: string;
}

export interface PostResponse {
  id: string;
  topic_id: string;
  title: string;
  slug: string;
  body: string;
  created_at: string;
}

export interface CommentResponse {
  id: string;
  post_id: string;
  body: string;
  created_at: string;
}

export interface SearchResultResponse {
  id: string;
  type: string;
  title: string;
  snippet: string;
  score: number;
}

export interface QuillEvent {
  type: string;
  payload: Record<string, unknown>;
}
