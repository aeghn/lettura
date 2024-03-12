import { useEffect } from "react";
import { Outlet, NavLink, useMatch, useNavigate } from "react-router-dom";
import { DndProvider } from "react-dnd";
import { HTML5Backend } from "react-dnd-html5-backend";
import clsx from "clsx";
import { Search, PlusCircle, Settings, FolderPlus, RefreshCw } from "lucide-react";
import { ChannelList } from "../../components/Subscribes";
import { useBearStore } from "@/stores";
import { RouteConfig } from "@/config";

import { TooltipBox } from "@/components/TooltipBox";
import { SpaceSwitcher } from "@/components/SpaceSwitcher";
import { useModal } from "@/components/Modal/useModal";
import { AddFeedChannel } from "@/components/AddFeed";
import { AddFolder } from "@/components/AddFolder";
import { useRefresh } from "@/components/Subscribes/useRefresh";
import { Icon } from "@/components/Icon";
import { SettingDialog } from "../Setting/SettingDialog";
import { ThreeStageButton } from "@/components/ui/switch-3";

const spaces = [
  {
    label: "Lettura",
    route: RouteConfig.LOCAL_TODAY,
    // icon: ;
  },
  // {
  //   label: "FreshRSS",
  //   route: RouteConfig.SERVICE_FRESHRSS,
  //   // icon: ;
  // },
];

export function LocalPage() {
  const navigate = useNavigate();
  const matched = useMatch(RouteConfig.LOCAL);
  const store = useBearStore((state) => ({
    feed: state.feed,
    filterValue: state.filterValue,
    setFilterValue: state.setFilterValue,
  }));
  const [feedList, setFeedList, getFeedList, refreshing, setRefreshing, done, setDone, startRefresh] = useRefresh();
  const [addFolderDialogStatus, setAddFolderDialogStatus] = useModal();
  const [editFolderDialogStatus, setEditFolderDialogStatus] = useModal();

  const iconSize = 16;
  const props = {
    name1: "Starred",
    name2: "Unread",
    name3: "All",
    onStageOne: () => {
      store.setFilterValue(0);
    },
    onStageTwo: () => {
      store.setFilterValue(1);
    },
    onStageThree: () => {
      store.setFilterValue(2);
    },
    initValue: 1,
    iconSize: iconSize,
  };

  useEffect(() => {
    if (store.feed && matched) {
      const { feed } = store;

      navigate(
        `${RouteConfig.LOCAL_FEED.replace(/:uuid/, feed.uuid)}?feedUuid=${feed.uuid}&feedUrl=${feed.feed_url}&type=${
          feed.item_type
        }`,
        {
          replace: true,
        }
      );
    }
  }, [matched]);

  return (
    <div className="flex flex-row h-full bg-canvas">
      <div
        className="relative flex h-full w-[var(--app-feedlist-width)] select-none flex-col text-[hsl(var(--foreground))]
  "
      >
        <div className="px-2 pb-3 text-sm flex-col flex">
          <div className="flex flex-row  items-center rounded-sm justify-around">
            <TooltipBox content="Search content" side="bottom" className="sidebar-item">
              <NavLink to={RouteConfig.SEARCH}>
                <Search size={iconSize} />
              </NavLink>
            </TooltipBox>

            <TooltipBox content="Go to settings" side="bottom" className="sidebar-item">
              {/* <NavLink
              to={RouteConfig.SETTINGS_GENERAL}
              className={({ isActive }) => {
                return clsx("sidebar-item", isActive ? "sidebar-item--active" : "");
              }}
            >
            </NavLink> */}
              <SettingDialog>
                <Settings size={iconSize} />
              </SettingDialog>
            </TooltipBox>

            <TooltipBox content="Add Folder" side="bottom" className="sidebar-item">
              <AddFolder
                action="add"
                dialogStatus={addFolderDialogStatus}
                setDialogStatus={setAddFolderDialogStatus}
                afterConfirm={getFeedList}
                afterCancel={() => {}}
                trigger={<FolderPlus size={iconSize} />}
              />
            </TooltipBox>

            <TooltipBox content="Add Feed" side="bottom" className={"sidebar-item"}>
              <AddFeedChannel>
                <PlusCircle size={iconSize} />
              </AddFeedChannel>
            </TooltipBox>

            <TooltipBox content="Update" side="bottom" className="sidebar-item">
              <button onClick={startRefresh}>
                <RefreshCw size={iconSize} className={`${refreshing ? "spinning" : ""}`} />
              </button>
            </TooltipBox>
          </div>
        </div>
        <DndProvider backend={HTML5Backend}>
          <ChannelList />
        </DndProvider>
        <div>
          <ThreeStageButton props={props} />
        </div>
      </div>

      <Outlet />
    </div>
  );
}
