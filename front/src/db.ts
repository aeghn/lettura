export interface FeedResItem {
  item_type: string;
  children?: FeedResItem[];
  uuid: string;
  title: string;
  link: string;
  feed_url: string;
  folder_name?: string;
  folder?: string;
  logo?: string;
  description: string;
  pub_time?: Date;
  health_status: number;
  failure_reason: string;
  unread: number;
  sort: number;
  create_time?: Date;
  update_time?: Date;
  last_sync_date?: Date;
  folder_uuid?: string | null;
  is_expanded?: boolean;
  parent_id: String;
  sync_interval_sec: number;
}

export interface ArticleResItem {
  author?: string;
  id: string;
  feed_id: string;
  feed_title: string;
  feed_url: string;
  title: string;
  link: string;
  image: string;
  description: string;
  content?: string;
  pub_time: string;
  create_time: string;
  is_read: boolean;
  is_starred?: boolean;
  media_object?: string;
  cached_content?: string;
}

export interface Folder {
  uuid: string;
  name: string;
}

export interface FeedLog {
  feed_id: String;
  last_pub_date?: string;
  healthy: boolean;
  log: String;
  create_time: string;
  update_time: string;
}
