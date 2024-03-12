import React, { ForwardedRef, useEffect, useState } from "react";
import { formatDistanceToNow, parseISO } from "date-fns";
import classnames from "classnames";
import { useBearStore } from "@/stores";
import { getChannelFavicon } from "@/helpers/parseXML";
import { ArticleResItem } from "@/db";
import { ArticleReadStatus } from "@/typing";
import { replaceImgUrl } from "@/helpers/domConvter";

export const ArticleItem = React.forwardRef((props: { article: ArticleResItem }, ref: ForwardedRef<HTMLLIElement>) => {
  const store = useBearStore((state) => ({
    updateArticleStatus: state.updateArticleStatus,
    article: state.article,
    setArticle: state.setArticle,
    feed: state.feed,
  }));
  const { article } = props;
  const [highlight, setHighlight] = useState<boolean>();
  const [readStatus, setReadStatus] = useState(article.is_read);

  const updateCurrentArticle = (article: any) => {
    if (article.is_read === false) {
      setReadStatus(true);
    }

    store.updateArticleStatus({ ...article }, true);
    store.setArticle(article);
  };

  const handleClick = async (e: React.MouseEvent) => {
    updateCurrentArticle(article);
  };

  const ico = getChannelFavicon(article.feed_url);

  useEffect(() => {
    setReadStatus(article.is_read);
  }, [article.is_read]);

  useEffect(() => {
    setHighlight(store.article?.id === article.id);
  }, [store.article, article]);

  return (
    <li
      className={classnames(
        "list-none rounded-sm p-3 pl-6 grid gap-1 relative select-none",
        "group hover:bg-accent hover:cursor-pointer",
        {
          "text-[hsl(var(--foreground)_/_80%)]": readStatus === true,
          "bg-primary text-accent hover:bg-primary": highlight,
        }
      )}
      onClick={handleClick}
      ref={ref}
      id={article.id}
    >
      {readStatus === false && <div className="absolute left-2 top-4 w-2 h-2 rounded-full bg-primary" />}
      <div
        className={classnames(
          `${highlight ? "text-article-active-headline" : "text-article-headline"}`,
          "font-bold text-sm group-hover:text-article-active-headline break-all"
        )}
      >
        {article.title}
      </div>
      <div
        className={classnames(
          "text-xs line-clamp-2 break-all",
          "text-article-paragraph group-hover:text-article-active-paragraph",
          {
            "text-article-active-paragraph": highlight,
          }
        )}
      >
        {(article.description || "").replace(/<[^<>]+>/g, "")}
      </div>
      <div
        className={classnames(
          "flex justify-between items-center text-xs text-article-paragraph group-hover:text-article-active-paragraph",
          {
            "text-article-active-paragraph": highlight,
          }
        )}
      >
        <div className="flex items-center">
          <img src={replaceImgUrl(store.feed?.logo || ico)} alt="" className="rounded w-4 mr-1" />
          <span className="max-w-[146px] overflow-hidden text-ellipsis mr-1 whitespace-nowrap">
            {article.author || article.feed_title}
          </span>
        </div>
        <div className="whitespace-nowrap">
          {formatDistanceToNow(parseISO(article.pub_time), {
            includeSeconds: true,
            addSuffix: true,
          })}
        </div>
      </div>
    </li>
  );
});
