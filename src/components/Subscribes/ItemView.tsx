import { FC, useLayoutEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";
import clsx from "clsx";
import { RouteConfig } from "@/config";
import { FeedResItem } from "@/db";
import { useBearStore } from "@/stores";
import { getChannelFavicon } from "@/helpers/parseXML";
import { NiceFolderIcon } from "../NiceFolderIcon";

export interface CardProps {
  uuid: any;
  text: string;
  index: number;
  feed: FeedResItem;
  className?: String;
  children?: any;
  arrow?: React.ReactNode;
  isActive: Boolean;
  isExpanded: Boolean;
  level?: number;
  toggleFolder: (uuid: string) => void;
}

export const ItemView: FC<CardProps> = ({
  uuid,
  text,
  feed,
  index,
  isExpanded,
  toggleFolder,
  ...props
}) => {
  const { isActive, level } = props;
  const navigate = useNavigate();
  const store = useBearStore((state) => ({
    feed: state.feed,
    setFeed: state.setFeed,
    getFeedList: state.getFeedList,
    setFeedContextMenuTarget: state.setFeedContextMenuTarget,
    feedContextMenuTarget: state.feedContextMenuTarget,
    feedContextMenuStatus: state.feedContextMenuStatus,
  }));

  const handleToggle = () => {
    if (feed.item_type === "folder") {
      toggleFolder(uuid);
    }
  };

  const { unread = 0, link, logo } = feed;
  const ico = logo || getChannelFavicon(link);

  function renderNiceFolder(isActive: Boolean, isExpanded: Boolean) {
    let folderStatus: string;

    if (isExpanded) {
      folderStatus = "open";
    } else if (isActive) {
      folderStatus = "active";
    } else {
      folderStatus = "close";
    }

    return <NiceFolderIcon status={folderStatus} onClick={handleToggle} />;
  }

  return (
    <div
      key={feed.title}
      onClick={() => {
        store.setFeed(feed);
        navigate(
          `${RouteConfig.LOCAL_FEED.replace(/:uuid/, feed.uuid)}?feedUuid=${
            feed.uuid
          }&feedUrl=${feed.feed_url}&type=${feed.item_type}`,
        );
      }}
    >
      <div
        className={clsx("sidebar-item", {
          "sidebar-item--active": isActive,
          "shadow-[inset_0_0_0_2px_var(--color-primary)]":
            store.feedContextMenuStatus &&
            store.feedContextMenuTarget &&
            store.feedContextMenuTarget.uuid === feed.uuid,
          "pl-9": level === 2,
        })}
        onContextMenu={() => {
          store.setFeedContextMenuTarget(feed);
        }}
      >
        {feed.item_type === "folder" && (
          <div>{renderNiceFolder(isActive, isExpanded)}</div>
        )}
        {feed.link && (
          <img src={ico} className="mr-2 h-5 w-5 rounded" alt={feed.title} />
        )}
        <span
          className={clsx(
            "shrink grow basis-[0%] overflow-hidden text-ellipsis whitespace-nowrap text-sm",
          )}
        >
          {feed.title}
        </span>
        {unread > 0 && (
          <span
            className={clsx(
              "-mr-1 h-4 min-w-[1rem] text-center text-[10px] leading-4",
            )}
          >
            {unread}
          </span>
        )}
      </div>
      {props.children}
    </div>
  );
};
