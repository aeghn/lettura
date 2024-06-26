import React, { useCallback, useEffect, useRef } from "react";
import { NavLink, useMatch, useNavigate } from "react-router-dom";
import {
  RefreshCw,
  Coffee,
  Haze,
  FolderPlus,
  CheckCheck,
  Pencil,
  Trash2,
  Rss,
  Image,
  ExternalLink,
  BellOff,
  FileText,
  Link,
  Link2,
  Star,
} from "lucide-react";
import classNames from "classnames";
import { RouteConfig } from "@/config";
import { FeedResItem, Folder } from "@/db";
import * as dataAgent from "@/helpers/dataAgent";
import { busChannel } from "@/helpers/busChannel";
import { useBearStore } from "@/stores";
import { AddFolder } from "../AddFolder";

import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { Icon } from "../Icon";
import { DialogUnsubscribeFeed } from "@/layout/Setting/Content/DialogUnsubscribeFeed";
import { useModal } from "../Modal/useModal";
import { DialogEditFeed } from "@/layout/Setting/Content/DialogEditFeed";
import { useQuery } from "@/helpers/parseXML";
import { useRefresh } from "./useRefresh";
import { ListContainer } from "./ListContainer";
import { copyText } from "@/helpers/copyText";
import { toast } from "sonner";
import { DialogDeleteFolder } from "@/layout/Setting/Content/DialogDeleteFolder";
import { loadFeed } from "@/hooks/useLoadFeed";
import clsx from "clsx";
import { useScrollTop } from "@/hooks/useScrollTop";
import { ArticleReadStatus } from "@/typing";

type NavClass = {
  isActive: boolean;
  isPending: boolean;
  isTransitioning: boolean;
};

const ChannelList = (): JSX.Element => {
  const isToday = useMatch(RouteConfig.LOCAL_TODAY);
  const isAll = useMatch(RouteConfig.LOCAL_ALL);
  const isStarred = useMatch(RouteConfig.LOCAL_STARRED);
  const navigate = useNavigate();
  const [editFolderDialogStatus, setEditFolderDialogStatus] = useModal();
  const [deleteFolderStatus, setDeleteFolderStatus] = useModal();
  const [editFeedStatus, setEditFeedStatus] = useModal();
  const [showStatus, setModalStatus] = useModal();
  const [feedList, setFeedList, getFeedList, refreshing, setRefreshing, done, setDone, startRefresh] = useRefresh();
  const store = useBearStore((state) => ({
    feed: state.feed,
    setFeed: state.setFeed,
    updateFeed: state.updateFeed,
    feedContextMenuTarget: state.feedContextMenuTarget,
    setFeedContextMenuTarget: state.setFeedContextMenuTarget,
    setFeedContextMenuStatus: state.setFeedContextMenuStatus,
    articleList: state.articleList,
    setArticleList: state.setArticleList,

    setViewMeta: state.setViewMeta,
    collectionMeta: state.collectionMeta,
    initCollectionMetas: state.initCollectionMetas,
    syncArticles: state.syncArticles,
  }));

  const [, , feedUuid] = useQuery();
  const [scrollTop, scrollTopProps] = useScrollTop();

  useEffect(() => {
    store.initCollectionMetas();
  }, []);

  useEffect(() => {
    getFeedList();
    const unsubscribeGetChannels = busChannel.on("getChannels", () => {
      getFeedList();
    });

    return () => {
      unsubscribeGetChannels();
    };
  }, []);

  const reloadFeedIcon = (feed: FeedResItem | null) => {
    feed &&
      dataAgent.updateIcon(feed.uuid, feed.link).then((res) => {
        feed.logo = res;
      });
  };

  const reloadFeedData = (feed: FeedResItem | null) => {
    if (feed) {
      loadFeed(
        feed,
        store.syncArticles,
        () => {
          // TODO: get article List
        },
        () => {}
      );
    }
  };

  const markAllRead = () => {
    if (store.feedContextMenuTarget) {
      const { uuid } = store.feedContextMenuTarget;

      toast.promise(dataAgent.markAllRead({ uuid, isToday: !!isToday, isAll: !!isAll }), {
        loading: "Loading...",
        success: (data) => {
          getFeedList();
          store.initCollectionMetas();

          if (store.feed?.uuid === uuid) {
            store.setArticleList(
              store.articleList.map((_) => {
                _.is_read = true;
                return _;
              })
            );
          }
          return `Done!😀`;
        },
        error: "Error🤢",
      });
    }
  };

  useEffect(() => {
    feedList.forEach((feed) => {
      if (feed.uuid === feedUuid) {
        store.setFeed(feed);
      }
    });
  }, [feedList]);

  const listRef = useRef<HTMLDivElement>(null);
  const handleListScroll = useCallback(() => {
    if (listRef.current) {
      const scrollTop = listRef.current.scrollTop;

      if (scrollTop > 0) {
        listRef.current?.parentElement?.classList.add("is-scroll");
      } else {
        listRef.current?.parentElement?.classList.remove("is-scroll");
      }
    }
  }, []);

  const handleContextMenuChange = (status: boolean) => {
    store.setFeedContextMenuStatus(status);
  };

  const afterDeleteFolder = () => {
    if (store.feedContextMenuTarget) {
      const { uuid } = store.feedContextMenuTarget;
      if (store.feed?.uuid === uuid) {
        store.setArticleList([]);
      }
      store.setFeedContextMenuTarget(null);
    }

    getFeedList();
  };

  const afterUnsubscribeFeed = () => {
    if (store.feedContextMenuTarget) {
      const { uuid } = store.feedContextMenuTarget;
      if (store.feed?.uuid === uuid) {
        store.setArticleList([]);
      }
      store.setFeedContextMenuTarget(null);
    }

    getFeedList();
  };

  return (
    <>
      <div
        {...scrollTopProps}
        className={clsx("height-[calc(100% - var(--app-toolbar-height))] flex-1 overflow-y-auto pb-2 pl-2 pr-1", {
          "border-t": scrollTop > 0,
        })}
        ref={listRef}
      >
        <h2 className="mb-2 mt-6 px-4 text-lg font-semibold tracking-tight">Collections</h2>
        <div>
          <div
            onClick={() => {
              store.setFeed(null);
              store.setViewMeta({
                title: "Today",
                isToday: true,
                isAll: false,
              });
            }}
          >
            <NavLink
              to={RouteConfig.LOCAL_TODAY}
              className={({ isActive }) => {
                return clsx("sidebar-item", isActive ? "sidebar-item--active" : "");
              }}
            >
              <Haze size={16} />
              <span className="shrink grow basis-[0%] overflow-hidden text-ellipsis whitespace-nowrap text-sm">
                Today
              </span>
              {store.collectionMeta.today.unread > 0 && (
                <span
                  className={classNames(
                    "-mr-1 h-4 min-w-[1rem] px-1 flex items-center justify-center text-[10px] leading-4 rounded",
                    "bg-secondary text-secondary-foreground"
                  )}
                >
                  {store.collectionMeta.today.unread}
                </span>
              )}
            </NavLink>
          </div>
          <div
            onClick={() => {
              store.setFeed(null);
              store.setViewMeta({
                title: "All Items",
                isToday: false,
                isAll: true,
              });
            }}
          >
            <NavLink
              to={RouteConfig.LOCAL_ALL}
              className={({ isActive }) => {
                return clsx("sidebar-item", isActive ? "sidebar-item--active" : "");
              }}
            >
              <Coffee size={16} />
              <span className="shrink grow basis-[0%] overflow-hidden text-ellipsis whitespace-nowrap text-sm">
                All Items
              </span>
              {store.collectionMeta.total.unread > 0 && (
                <span
                  className={classNames(
                    "-mr-1 h-4 min-w-[1rem] px-1 flex items-center justify-center text-[10px] leading-4 rounded",
                    "bg-secondary text-secondary-foreground"
                  )}
                >
                  {store.collectionMeta.total.unread}
                </span>
              )}
            </NavLink>
          </div>
        </div>
        <h2 className="mb-2 mt-6 px-4 text-lg font-semibold tracking-tight">Feeds</h2>
        <ContextMenu onOpenChange={handleContextMenuChange}>
          <ContextMenuTrigger>
            <ListContainer />
          </ContextMenuTrigger>
          <ContextMenuContent key={store.feedContextMenuTarget?.uuid || "0"}>
            <ContextMenuItem
              onClick={() => {
                markAllRead();
              }}
            >
              <CheckCheck size={14} className="mr-2" /> Mark all as read
            </ContextMenuItem>
            <ContextMenuSeparator />
            {store.feedContextMenuTarget?.item_type === "folder" && (
              <>
                <ContextMenuItem onSelect={() => setEditFolderDialogStatus(true)}>
                  <Pencil size={14} className="mr-2" /> Edit folder
                </ContextMenuItem>
                <ContextMenuSeparator />
                <ContextMenuItem onClick={() => setDeleteFolderStatus(true)} className="text-red-600">
                  <Trash2 size={14} className="mr-2" /> Delete folder
                </ContextMenuItem>
              </>
            )}
            {store.feedContextMenuTarget && (
              <>
                {store.feedContextMenuTarget?.item_type !== "folder" && (
                  <>
                    <ContextMenuItem
                      onClick={() => store.feedContextMenuTarget?.link && open(store.feedContextMenuTarget?.link)}
                    >
                      <ExternalLink size={14} className="mr-2" /> Open Home Page
                    </ContextMenuItem>
                    <ContextMenuSeparator />
                    <ContextMenuItem
                      onClick={() =>
                        store.feedContextMenuTarget?.feed_url &&
                        copyText(store.feedContextMenuTarget?.feed_url).then(() =>
                          toast.message("Current URL copied to clipboard", {
                            description: "Paste it wherever you like",
                          })
                        )
                      }
                    >
                      <Link size={14} className="mr-2" /> Copy Feed URL
                    </ContextMenuItem>
                    <ContextMenuItem
                      onClick={() =>
                        store.feedContextMenuTarget?.link &&
                        copyText(store.feedContextMenuTarget?.link).then(() =>
                          toast.message("Current URL copied to clipboard", {
                            description: "Paste it wherever you like",
                          })
                        )
                      }
                    >
                      <Link2 size={14} className="mr-2" /> Copy Home Page URL
                    </ContextMenuItem>
                    <ContextMenuSeparator />
                    <ContextMenuItem onClick={() => reloadFeedIcon(store.feedContextMenuTarget)}>
                      <Image size={14} className="mr-2" /> Reload icon
                    </ContextMenuItem>
                    <ContextMenuItem onClick={() => reloadFeedData(store.feedContextMenuTarget)}>
                      <Rss size={14} className="mr-2" /> Reload feeds
                    </ContextMenuItem>
                    <ContextMenuSeparator />
                    <ContextMenuItem onClick={() => setEditFeedStatus(true)}>
                      <FileText size={14} className="mr-2" /> View detail
                    </ContextMenuItem>
                    <ContextMenuSeparator />
                    <ContextMenuItem onClick={() => setModalStatus(true)} className="text-red-600">
                      <BellOff size={14} className="mr-2" /> Unsubscribe
                    </ContextMenuItem>
                  </>
                )}
              </>
            )}
          </ContextMenuContent>
        </ContextMenu>
        <DialogUnsubscribeFeed
          feed={store.feedContextMenuTarget}
          dialogStatus={showStatus}
          setDialogStatus={setModalStatus}
          afterConfirm={afterUnsubscribeFeed}
          afterCancel={() => store.setFeedContextMenuTarget(null)}
        />
        <DialogDeleteFolder
          folder={store.feedContextMenuTarget as FeedResItem & Folder}
          dialogStatus={deleteFolderStatus}
          setDialogStatus={setDeleteFolderStatus}
          afterConfirm={afterDeleteFolder}
          afterCancel={() => store.setFeedContextMenuTarget(null)}
        />
        <DialogEditFeed
          feed={store.feedContextMenuTarget}
          dialogStatus={editFeedStatus}
          setDialogStatus={setEditFeedStatus}
          afterConfirm={getFeedList}
          afterCancel={() => store.setFeedContextMenuTarget(null)}
        />
        <AddFolder
          action="edit"
          folder={store.feedContextMenuTarget}
          dialogStatus={editFolderDialogStatus}
          setDialogStatus={setEditFolderDialogStatus}
          afterConfirm={getFeedList}
          afterCancel={() => store.setFeedContextMenuTarget(null)}
        />
      </div>
      {refreshing && (
        <div className="sticky bottom-0 left-0 right-0 p-2 text-right">
          <span className="mr-3 text-xs">Syncing...</span>
          <span className="text-foreground text-xs">
            {done}/{feedList.length}
          </span>
        </div>
      )}
    </>
  );
};

export { ChannelList };
