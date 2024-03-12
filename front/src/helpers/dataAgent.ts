import { ArticleResItem, FeedLog, FeedResItem, Folder } from "../db";
import { request } from "@/helpers/request";
import { AxiosRequestConfig, AxiosResponse } from "axios";
import { replaceImg } from "./domConvter";

export const getChannels = async (
  filter: any
): Promise<AxiosResponse<{ list: (FeedResItem & { parent_id: String })[] }>> => {
  return request.get("feeds", {
    params: {
      filter,
    },
  });
};

export const getFeeds = async (): Promise<AxiosResponse<FeedResItem[]>> => {
  return request.get("subscribes").then((subs: AxiosResponse<FeedResItem[]>) => {
    const map = new Map<String, FeedResItem>();
    for (const sub of subs.data) {
      map.set(sub.uuid, sub);
      sub.children = [];
    }

    for (const sub of subs.data) {
      if (map.has(sub.parent_id)) {
        map.get(sub.parent_id)?.children?.push(sub);
        map.delete(sub.uuid);
      }
    }
    subs.data = [...map.values()].sort((a, b) => a.sort - b.sort);
    console.log("data: ", subs);
    return subs;
  });
};

export const createFolder = async (name: string): Promise<AxiosResponse<FeedResItem[]>> => {
  return request.post(`create-folder`, { name });
};

export const updateFolder = async (uuid: string, name: string): Promise<AxiosResponse<number>> => {
  return request.post("update-folder", { uuid, name });
};

export const getFolders = async (): Promise<AxiosResponse<Folder[]>> => {
  return request.post("get-folders", {});
};

export const updateFeedSort = async (
  sorts: {
    item_type: string;
    uuid: string;
    folder_uuid: string;
    sort: number;
  }[]
): Promise<any> => {
  return request.post("update-feed-sort", sorts);
};

export const moveChannelIntoFolder = async (channelUuid: string, folderUuid: string, sort: number): Promise<any> => {
  return request.post("move-channel-into-folder", {
    channel_uuid: channelUuid,
    folder_uuid: folderUuid,
    sort,
  });
};

/**
 * 删除频道
 * @param {String} uuid  channel 的 uuid
 */
export const deleteChannel = async (uuid: string) => {
  return request.delete(`feeds/${uuid}`);
};

export const deleteFolder = async (uuid: string) => {
  return request.post(`delete-folder/${uuid}`, {});
};

export const getArticleList = async (filter: any) => {
  const req = request.get("articles", {
    params: {
      ...filter,
    },
  });

  return req;
};

export const fetchFeed = async (url: string): Promise<AxiosResponse<FeedResItem>> => {
  return request.post(`fetch-feed`, { url });
};

export const subscribeFeed = async (url: string): Promise<[FeedResItem, number]> => {
  return request.post(`add-feed`, { url }).then((rest: any) => {
    if (!rest) {
      throw new Error("Response is empty");
    }

    return [rest.feed as FeedResItem, rest.article_count as number];
  });
};

export const updateFeedSyncInterval = async (id: string, interval: number): Promise<AxiosResponse<FeedResItem>> => {
  return request.post(`update-feed-sync-interval`, null, {
    params: {
      id,
      interval,
    },
  });
};

export const syncFeed = async (
  feed_type: string,
  uuid: string
): Promise<AxiosResponse<{ [key: string]: [string, number, string] }>> => {
  return request.get(`/feeds/${uuid}/sync`);
};

export const getFeedLog = async (uuid: string): Promise<AxiosResponse<[FeedLog]>> => {
  return request.get(`/feed-log/${uuid}`);
};

export const getUnreadTotal = async (value: number): Promise<AxiosResponse<{ [key: string]: number }>> => {
  return request.get("unread-total", {
    params: {
      filter: value,
    },
  });
};

export const getCollectionMetas = async (): Promise<
  AxiosResponse<{
    [key: string]: number;
  }>
> => {
  return request.get("collection-metas");
};

export const updateArticleReadStatus = async (article_uuid: string, is_read: boolean) => {
  return request.post(`/articles/${article_uuid}/read`, {
    is_read,
  });
};

export const updateArticleStarStatus = async (article_uuid: string, is_starred: boolean) => {
  return request.post(`/articles/${article_uuid}/star`, {
    is_starred,
  });
};

export const markAllRead = async (body: {
  uuid?: string;
  isToday?: boolean;
  isAll?: boolean;
}): Promise<AxiosResponse<number>> => {
  return request.post("/mark-all-as-read", body);
};

export const getUserConfig = async (): Promise<any> => {
  return request.get("/user-config");
};

export const updateUserConfig = async (cfg: any): Promise<any> => {
  return request.post("/user-config", cfg);
};

export const updateProxy = async (cfg: LocalProxy): Promise<any> => {
  return request.post("update-proxy", {
    ip: cfg.ip,
    port: cfg.port,
  });
};

export const updateThreads = async (threads: number): Promise<any> => {
  return request.post("update_threads", { threads });
};

export const updateTheme = async (theme: string): Promise<any> => {
  return request.post("update-theme", { theme });
};

export const updateInterval = async (interval: number): Promise<any> => {
  return request.post("update-interval", { interval });
};

export const initProcess = async (): Promise<any> => {
  return request.post("init-process", {});
};

export const getArticleDetail = async (
  uuid: string,
  config: AxiosRequestConfig
): Promise<AxiosResponse<ArticleResItem>> => {
  return request.get(`articles/${uuid}`, config).then((rsp) => {
    const article = rsp.data;

    article.content = replaceImg(article.content);
    article.description = replaceImg(article.description);

    console.log("article, ", article);
    return rsp;
  });
};

export const getBestImage = async (url: String): Promise<AxiosResponse<string>> => {
  return request.get(`image-proxy`, {
    params: {
      url,
    },
  });
};

export const getPageSources = async (url: string): Promise<AxiosResponse<string>> => {
  return request.get(`article-proxy`, {
    params: {
      url,
    },
  });
};

export const updateIcon = async (uuid: String, url: string): Promise<any> => {
  return request.get(`update-icon/${uuid}/${url}`);
};
