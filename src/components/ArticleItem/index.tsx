import React, { useEffect, useState } from "react";
import Dayjs from "dayjs";
import { useStore } from "../../hooks/useStore";
import styles from "./articleitem.module.scss";
import { busChannel } from "../../helpers/busChannel";
import * as dataAgent from "../../helpers/dataAgent";

export const ArticleItem = React.memo((props: any) => {
  const { article, onSelect, highlight } = props;
  console.log("%c Line:10 🍞 article", "color:#42b983", article);
  const store = useStore();
  const [readStatus, setReadStatus] = useState(article.read_status);

  const handleClick = async (e: any) => {
    if (onSelect) {
      onSelect(article);
    }

    if (article.read_status === 1) {
      const res = await dataAgent.updateArticleReadStatus(article.uuid, 2);
      if (res) {
        busChannel.emit("updateChannelUnreadCount", {
          uuid: article.channel_uuid,
          action: "decrease",
          count: 1,
        });

        setReadStatus(true);
      }
    }

    store.setArticle(article);
  };

  useEffect(() => {
    setReadStatus(article.read_status)
    console.log("%c Line:37 🍞 article.read_status", "color:#fca650", article.read_status);
  }, [article.read_status])

  return (
    <li
      className={`${styles.item} ${readStatus === 2 ? styles.read : ""} ${
        highlight ? styles.current : ""
      }`}
      onClick={handleClick}
      aria-hidden="true"
    >
      {(readStatus === 1) && <div className={styles.dot} />}
      <div className={styles.title}>
        <div className={styles.titleText}>
          {highlight} {article.title}
        </div>
      </div>
      <div className={styles.description}>
        {(article.description || "").replace(/<[^<>]+>/g, "")}
      </div>
      <div className={styles.meta}>
        <div>{article.author}</div>
        <div className={styles.date}>
          {Dayjs(article.pub_date).format("YYYY-MM-DD HH:mm")}
        </div>
      </div>
    </li>
  );
});
