import useSWRInfinite from "swr/infinite";
import { useBearStore } from "@/stores";
import { request } from "@/helpers/request";
import { useMatch } from "react-router-dom";
import { RouteConfig } from "@/config";
import { omit, throttle } from "lodash";
import { ArticleResItem } from "@/db";

const PAGE_SIZE = 20;

export interface UseArticleProps {
  feedUuid?: string;
  type?: string;
}

export function useArticle(props: UseArticleProps) {
  const { feedUuid, type } = props;
  const isToday = useMatch(RouteConfig.LOCAL_TODAY);
  const isAll = useMatch(RouteConfig.LOCAL_ALL);
  const isStarred = useMatch(RouteConfig.LOCAL_STARRED);
  const store = useBearStore((state) => ({
    currentFilter: state.currentFilter,
    updateArticleStatus: state.updateArticleStatus,
    filterValue: state.filterValue,
  }));

  const query = omit({
    limit: PAGE_SIZE,
    feed_id: feedUuid,
    item_type: type,
    is_today: isToday ? true : false,
    is_all: isAll ? true : false,
    is_starred: store.filterValue === 0,
    is_read: store.filterValue === 1 ? false : null,
  });

  console.log("%c Line:29 🍖 query", "color:#ea7e5c", query);

  const getKey = (pageIndex: number, previousPageData: any) => {
    if (previousPageData && !previousPageData.list?.length) return null; // 已经到最后一页
    console.log("pageIndex + 1", pageIndex + 1);
    return {
      ...query,
      cursor: pageIndex + 1,
    }; // SWR key
  };
  const { data, isLoading, size, mutate, setSize } = useSWRInfinite(
    getKey,
    (q) =>
      request
        .get("/articles", {
          params: { ...q },
        })
        .then((res) => res.data),
    {
      revalidateIfStale: false,
      revalidateOnFocus: false,
      revalidateOnReconnect: true,
    }
  );

  const list = data ? data.reduce((acu, cur) => acu.concat(cur.list || []), []) : [];
  const articles: ArticleResItem[] = list ? [].concat(list) : [];
  const isEmpty = !isLoading && list.length === 0;
  const isReachingEnd = isEmpty || (data && data[data.length - 1]?.list?.length < PAGE_SIZE);

  return {
    articles,
    isLoading,
    mutate,
    size,
    setSize,
    isEmpty,
    isReachingEnd,
    isToday: !!isToday,
    isAll: !!isAll,
    isStarred: !!isStarred,
  };
}
