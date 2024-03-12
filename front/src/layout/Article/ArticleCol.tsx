import React, {
  ForwardedRef,
  useCallback,
  useEffect,
  useImperativeHandle,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { useParams } from "react-router-dom";
import { ArticleList } from "@/components/ArticleList";
import { useBearStore } from "@/stores";
import * as dataAgent from "@/helpers/dataAgent";

import {
  Filter,
  CheckCheck,
  RefreshCw,
  LucidePiano,
  LucideAArrowDown,
  LucideCircle,
  LucideBookDown,
  LucideBook,
} from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

import { Icon } from "@/components/Icon";
import { TooltipBox } from "@/components/TooltipBox";
import { useArticle } from "./useArticle";
import { loadFeed } from "@/hooks/useLoadFeed";
import { ArticleReadStatus } from "@/typing";
import { useHotkeys } from "react-hotkeys-hook";
import { throttle } from "lodash";
import { ArticleResItem, FeedLog } from "@/db";
import { formatDistanceToNow, parseISO } from "date-fns";

export interface ArticleColRefObject {
  goNext: () => void;
  goPrev: () => void;
}

export const ArticleCol = React.memo(
  React.forwardRef<ArticleColRefObject, any>((props: { feedUuid?: string; type?: string }, listForwarded) => {
    const { feedUuid, type } = props;
    // @ts-ignore
    const params: { name: string } = useParams();
    const [isSyncing, setIsSyncing] = useState(false);
    const listRef = useRef<HTMLDivElement>(null);

    const store = useBearStore((state) => ({
      viewMeta: state.viewMeta,
      article: state.article,
      setArticle: state.setArticle,
      feed: state.feed,
      syncArticles: state.syncArticles,
      markArticleListAsRead: state.markArticleListAsRead,

      updateArticleStatus: state.updateArticleStatus,
      setHasMorePrev: state.setHasMorePrev,
      setHasMoreNext: state.setHasMoreNext,

      filterList: state.filterList,
      currentFilter: state.currentFilter,
      setFilter: state.setFilter,

      userConfig: state.userConfig,
    }));

    const [feedLog, setFeedLog] = useState<FeedLog>();

    const { articles, isLoading, size, mutate, setSize, isEmpty, isReachingEnd, isToday, isAll, isStarred } =
      useArticle({
        feedUuid,
        type,
      });

    const handleRefresh = () => {
      if (store.feed && store.feed.uuid) {
        setIsSyncing(true);
        loadFeed(
          store.feed,
          store.syncArticles,
          () => {
            mutate();
            setIsSyncing(false);
          },
          () => {
            setIsSyncing(false);
          }
        );
      }
    };

    const markAllRead = () => {
      return store.markArticleListAsRead(isToday, isAll).then(() => {
        mutate();
      });
    };

    const changeFilter = (id: any) => {
      if (store.filterList.some((_) => _.id === parseInt(id, 10))) {
        store.setFilter({
          ...store.filterList.filter((_) => _.id === parseInt(id, 10))[0],
        });
      }
    };

    function calculateItemPosition(direction: "up" | "down", article: ArticleResItem | null) {
      if (!article?.id) {
        return;
      }

      const $li = document.getElementById(article.id);
      const bounding = $li?.getBoundingClientRect();
      const winH = window.innerHeight;

      if ((direction === "up" || direction === "down") && bounding && bounding.top < 58) {
        const offset = 58 - bounding.top;
        const scrollTop = (listRef?.current?.scrollTop || 0) - offset;

        listRef?.current?.scrollTo(0, scrollTop);
      } else if ((direction === "up" || direction === "down") && bounding && bounding.bottom > winH) {
        const offset = bounding.bottom - winH;
        const scrollTop = (listRef?.current?.scrollTop || 0) + offset;

        console.log("🚀 ~ file: index.tsx:324 ~ ArticleContainer ~ scrollTop:", scrollTop);
        listRef?.current?.scrollTo(0, scrollTop);
      }
    }

    const goPreviousArticle = () => {
      let previousItem: ArticleResItem;
      let uuid = store.article?.id;

      for (let i = 0; i < articles.length; i++) {
        if (articles[i].id === uuid && i === 0) {
          store.setHasMorePrev(false);
          store.setHasMoreNext(true);

          break;
        }

        if (articles[i].id === uuid && i !== 0) {
          previousItem = articles[i - 1];
          previousItem.is_read = true;

          store.updateArticleStatus({ ...previousItem }, true);
          store.setArticle(previousItem);
          store.setHasMorePrev(true);
          store.setHasMoreNext(true);

          calculateItemPosition("up", previousItem);

          break;
        }
      }
    };

    const goNextArticle = () => {
      let nextItem: ArticleResItem = {} as ArticleResItem;
      let uuid = store.article?.id;

      if (!uuid) {
        return false;
      }

      for (let i = 0; i < articles.length; i++) {
        if (articles[i].id === uuid && i === articles.length) {
          return [true];
        }

        if (articles[i].id === uuid && i < articles.length - 1) {
          nextItem = articles[i + 1];
          break;
        }
      }

      if (!uuid && articles.length > 0) {
        nextItem = articles[0];
      }

      console.log("%c Line:162 🥟 articles", "color:#4fff4B", articles);
      console.log("%c Line:162 🍔 nextItem", "color:#4fff4B", nextItem);

      store.updateArticleStatus({ ...nextItem }, true);

      nextItem.is_read = true;
      store.setArticle(nextItem);

      calculateItemPosition("down", nextItem);

      return [false];
    };

    const goPrev = throttle(() => {
      console.warn("goPrev");
      goPreviousArticle();
    }, 300);

    const goNext = throttle(() => {
      console.warn("goNext");
      goNextArticle();
    }, 300);

    useImperativeHandle(listForwarded, () => {
      return {
        goNext,
        goPrev,
      };
    });

    useHotkeys("Down", goNext);
    useHotkeys("Up", goPrev);

    useEffect(() => {
      setFeedLog(undefined);
      if (store.feed?.uuid) {
        dataAgent.getFeedLog(store.feed.uuid).then((fls) => {
          setFeedLog(fls.data[0]);
        });
      }
    }, [store.feed]);

    return (
      <div className="w-4/12 shrink-0 basis-[var(--app-article-width)] border-r flex flex-[1_1_0%] flex-col h-full">
        <div className="h-[var(--app-toolbar-height)] grid grid-cols-[auto_1fr] items-center justify-between border-b">
          <div
            className="
            flex
            items-center
            px-3
            text-base
            font-bold
            w-full
            text-ellipsis
            overflow-hidden
            whitespace-nowrap
            text-article-headline
          "
          >
            {store.viewMeta ? store.viewMeta.title : ""}
          </div>
          <div className={"flex items-center justify-end px-2 space-x-0.5"}>
            {feedLog ? (
              <div className="text-xs px-1 mx-2">
                <div className="flex flex-row">Pull: {formatDistanceToNow(parseISO(feedLog.update_time))}</div>
                <div className="flex flex-row">
                  Pub: {feedLog.last_pub_date ? formatDistanceToNow(parseISO(feedLog.last_pub_date)) : null}
                </div>
              </div>
            ) : null}

            <TooltipBox content="Reload feed">
              <Icon onClick={handleRefresh}>
                <RefreshCw
                  size={20}
                  className={`${isSyncing ? "spinning" : "333"}`}
                  color={feedLog?.healthy ? "#33aa33" : "#aa3333"}
                />
              </Icon>
            </TooltipBox>
          </div>
        </div>
        <div className="relative flex-1 overflow-auto" ref={listRef}>
          <ArticleList
            articles={articles}
            title={params.name}
            type={type}
            feedUuid={feedUuid}
            isLoading={isLoading}
            isEmpty={isEmpty}
            isReachingEnd={isReachingEnd}
            size={size}
            setSize={setSize}
          />
        </div>
      </div>
    );
  })
);
