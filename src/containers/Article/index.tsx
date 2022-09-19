import React, { useCallback, useEffect, useRef, useState } from "react";
import { useLocation, useParams } from "react-router-dom";
import { ArticleList } from "../../components/ArticleList";
import { ArticleView } from "../../components/ArticleView";
import { getChannelFavicon, requestFeed } from "../../helpers/parseXML";
import { Button, Dropdown, Notification } from "@douyinfe/semi-ui";
import { Icon } from "../../components/Icon";
import * as dataAgent from "../../helpers/dataAgent";
import { useStore } from "../../hooks/useStore";
import styles from "./index.module.scss";
import { makeAllRead } from "../../helpers/dataAgent";

function useQuery() {
  return new URLSearchParams(useLocation().search);
}

export const ArticleContainer = (): JSX.Element => {
  // @ts-ignore
  const params: { name: string } = useParams();
  const store = useStore();
  const query = useQuery();
  const feedUrl = query.get("feedUrl");
  const channelId = query.get("channelId");
  const [current, setCurrent] = useState<any>(null);
  const [syncing, setSyncing] = useState(false);
  const listRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<HTMLDivElement>(null);

  const handleListScroll = useCallback(() => {
    if (listRef.current) {
      const scrollTop = listRef.current.scrollTop;

      if (scrollTop > 0) {
        listRef.current.classList.add("is-scroll");
      } else {
        listRef.current.classList.remove("is-scroll");
      }
    }
  }, [listRef.current]);

  const handleViewScroll = useCallback(() => {
    if (viewRef.current) {
      const scrollTop = viewRef.current.scrollTop;
      console.log("scrolling", scrollTop);

      if (scrollTop > 0) {
        viewRef.current.classList.add("is-scroll");
      } else {
        viewRef.current.classList.remove("is-scroll");
      }
    }
  }, [viewRef.current]);

  useEffect(() => {
    if (listRef.current) {
      const $list = listRef.current as HTMLDivElement;
      $list.addEventListener("scroll", handleListScroll);
    }

    if (viewRef.current) {
      const $list = viewRef.current as HTMLDivElement;
      $list.addEventListener("scroll", handleViewScroll);
    }

  }, []);

  const handleArticleSelect = useCallback((article: any) => {
    setCurrent(article);
  }, []);

  const syncArticles = () => {
    if (feedUrl) {
      setSyncing(true)
      requestFeed(feedUrl).then((res) => {
        if (res.channel && res.items) {
          const { items } = res;

          return dataAgent.bulkAddArticle(items)
            .then(() => {
              return dataAgent.updateCountWithChannel(feedUrl);
            })
        }
      }).finally(() => {
        setSyncing(false)
      });
    }
  };

  const handleRefresh = () => {
    syncArticles();
  };

  const markAllRead = () => {
    if (feedUrl) {
      return dataAgent.makeAllRead(feedUrl).then(() => {})
    }

    return Promise.resolve()
  };

  const changeFilter = (filter: any) => {
    store.setFilter(filter);
  };

  // function favoriteIt() {
  // }

  const openInBrowser = () => {
  };

  useEffect(() => {
    if (viewRef.current !== null) {
      viewRef.current.scroll(0, 0);
    }
  }, [current]);

  useEffect(() => {
    if (listRef.current !== null) {
      listRef.current.scroll(0, 0);
    }
  }, [channelId]);

  return (
    <div className={styles.article}>
      <div className={styles.list} ref={listRef}>
        <div className={`sticky-header ${styles.header}`}>
          <div className={styles.title}>
            {store.channel ? store.channel.title : ""}
          </div>
          <div className={styles.menu}>
            <Dropdown
              trigger="click"
              position="bottomLeft"
              clickToHide={true}
              render={
                <Dropdown.Menu>
                  {store.filterList.map((item) => {
                    return <Dropdown.Item key={item.id} onClick={() => changeFilter(item)}
                                          {...item.id === store.currentFilter.id ? { type: "primary" } : {}}>{item.title}</Dropdown.Item>;
                  })}
                </Dropdown.Menu>
              }
            >
              <Button>{store.currentFilter.title}</Button>
            </Dropdown>
            <Icon
              customClass={styles.menuIcon}
              name="checklist"
              onClick={markAllRead}
            />
            <Icon
              customClass={styles.menuIcon}
              name="refresh"
              onClick={handleRefresh}
            />
          </div>
        </div>
        {syncing && <div className={styles.syncingBar}>同步中</div>}
        <ArticleList
          title={params.name}
          channelId={channelId}
          feedUrl={feedUrl}
          onArticleSelect={handleArticleSelect}
        />
      </div>
      <div className={styles.mainView} ref={viewRef}>
        <div className={`sticky-header ${styles.viewHeader}`}>
          <div className={styles.viewMenu}>
            {/*<Icon*/}
            {/*  customClass={`${styles.menuIcon}`}*/}
            {/*  name="done"*/}
            {/*  onClick={favoriteIt}*/}
            {/*/>*/}
            {/*<Icon*/}
            {/*  customClass={`${styles.menuIcon}`}*/}
            {/*  name="radio_button_unchecked"*/}
            {/*  onClick={favoriteIt}*/}
            {/*/>*/}
            {/*<Icon*/}
            {/*  customClass={`${styles.menuIcon}`}*/}
            {/*  name="favorite"*/}
            {/*  onClick={favoriteIt}*/}
            {/*/>*/}
            <a target="_blank" rel="noreferrer" href={current && current.link}>
              <Icon
                customClass={`${styles.menuIcon}`}
                name="link"
                onClick={openInBrowser}
              />
            </a>
          </div>
        </div>
        <ArticleView article={current}/>
      </div>
    </div>
  );
};
