import Dexie, { Table } from "dexie";

export interface Podcast {
  id?: number;
  uuid: string;
  title: string;
  link: string;
  feed_url: string;
  feed_id: string;
  feed_title: string;
  description: string;
  pub_time: string;
  create_time: string;
  update_time: string;
  starred: number;
  mediaURL: string;
  mediaType: string;
  thumbnail: string;
  add_date: number;
}

export class MySubClassedDexie extends Dexie {
  podcasts!: Table<Podcast>;

  constructor() {
    super("Lettura");

    //@ts-ignore
    this.version(1).stores({
      podcasts:
        "++id, &uuid, title, link, feed_url, feed_id, feed_title, description, pub_time, create_time, update_time, starred, mediaURL, mediaType, thumbnail, add_date", // Primary key and indexed props
    });
  }
}

export const db = new MySubClassedDexie();
