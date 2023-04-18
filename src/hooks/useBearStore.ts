import { create } from "zustand";
import { subscribeWithSelector } from 'zustand/middleware';
import { Article, Channel } from "../db";
import { busChannel } from "../helpers/busChannel";
import * as dataAgent from "../helpers/dataAgent"

interface BearStore {
  channel: Channel | null;
  setChannel: (channel: Channel) => void;

  article: Article | null;
  setArticle: (article: Article) => void;
  articleList: Article[];
  setArticleList: (list: Article[]) => void;

  updateArticleAndIdx: (article: Article, idx?: number) => void;
  goPreviousArticle: any;
  goNextArticle: any;

  currentIdx: number;
  setCurrentIdx: (idx: number) => void;

  currentFilter: { id: number; title: string };
  filterList: { id: number; title: string }[];
  setFilter: any;
}

export const useBearStore = create<BearStore>()(subscribeWithSelector((set, get) => {
  return {
    channel: null,
    setChannel: (channel: Channel) => {
      set(() => ({
        channel: channel,
      }));
    },

    article: null,
    setArticle: (article: Article) => {
      set(() => ({
        article: article,
      }));
    },

    articleList: [],
    setArticleList: (list: Article[]) => {
      set(() => ({
        articleList: list,
      }));
    },

    updateArticleAndIdx: (article: Article, idx?: number) => {
      console.log('update Article and Idx', idx);
      let articleList = get().articleList;

      if (idx === undefined || idx <= 0) {
        idx = articleList.findIndex((item) => item.uuid === article.uuid);
      }

      console.log("%c Line:59 🍬 article.read_status", "color:#fca650", article.read_status);

      if (article.read_status === 1) {
        dataAgent.updateArticleReadStatus(article.uuid, 2).then((res) => {
          if (res) {
            busChannel.emit("updateChannelUnreadCount", {
              uuid: article.channel_uuid,
              action: "decrease",
              count: 1,
            });

            article.read_status = 2;

            set(() => ({
              article,
              currentIdx: idx,
            }))
          }
        });
      }

      if (idx) {
        set(() => ({
          article,
          currentIdx: idx,
        }))
      } else {
        set(() => ({
          article,
        }))
      }
    },

    goPreviousArticle(){
      let cur = -1;
      let currentIdx = get().currentIdx;
      let articleList = get().articleList;

      if (currentIdx <= 0) {
        cur = 0;
      } else {
        cur = currentIdx - 1;
      }

      get().updateArticleAndIdx(articleList[cur], cur);
    },

    goNextArticle() {
      console.log("%c Line:108 🍐 goNextArticle", "color:#6ec1c2", "goNextArticle");
      let cur = -1;
      let currentIdx = get().currentIdx;
      let articleList = get().articleList;

      if (currentIdx < articleList.length - 1) {
        cur = currentIdx + 1;
      }

      get().updateArticleAndIdx(articleList[cur], cur);
    },

    currentIdx: 0,
    setCurrentIdx: (idx: number) => {
      set(() => ({
        currentIdx: idx,
      }));
    },

    filterList: [
      {
        id: 0,
        title: "All",
      },
      {
        id: 1,
        title: "Unread",
      },
      {
        id: 2,
        title: "Read",
      },
    ],
    currentFilter: {
      id: 1,
      title: "Unread",
    },
    setFilter: (filter: {id: number, title: string}) => {
      set(() => ({
        currentFilter: filter
      }))
    }
  };
}));
