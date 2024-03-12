import React, { useCallback, useEffect, useRef, useState } from "react";
import clsx from "clsx";
import styles from "@/components/ArticleView/view.module.scss";
import Dayjs from "dayjs";
import { getChannelFavicon } from "@/helpers/parseXML";
import { useBearStore } from "@/stores";
import * as dataAgent from "@/helpers/dataAgent";
import xss, { getDefaultWhiteList } from "xss";
import linkifyStr from "linkify-string";
import { ArticleResItem } from "@/db";
import { YoutubeAdapter } from "./adpater/Youtube";
import { PodcastAdapter } from "./adpater/Podcast";
import { StarAndRead } from "@/layout/Article/StarAndRead";
import { Separator } from "../ui/separator";
import { TooltipBox } from "../TooltipBox";
import { Icon } from "../Icon";
import { Readability } from "@mozilla/readability";
import { replaceImgUrl } from "@/helpers/domConvter";
import { Loader2, Scroll, ScrollText } from "lucide-react";

function createMarkup(html: string) {
  return { __html: html };
}

function validateFeed(article: ArticleResItem, medias: any) {
  const { feed_url } = article;

  let isCommon = true;
  let isYoutube = false;
  let isPodcast = false;

  if (/youtube.com\/feeds\/videos.xml/.test(feed_url)) {
    isYoutube = true;
    isCommon = false;
  } else if (medias?.length > 0) {
    isPodcast = true;
    isCommon = false;
  }

  return {
    isCommon,
    isYoutube,
    isPodcast,
  };
}

export interface ArticleDetailProps {
  article: any;
}

export const ArticleDetail = (props: ArticleDetailProps) => {
  const { article } = props;
  const store = useBearStore((state) => ({
    feed: state.feed,
  }));
  const { pub_time: pub_time, feed_url } = article;
  const ico = getChannelFavicon(feed_url);
  const [pageContent, setPageContent] = useState("");
  const [medias, setMedias] = useState([]);
  const controller = new AbortController();

  const [renderFull, setRenderFull] = useState(false);
  const [originalContent, setOriginalContent] = useState();
  const [loadingFull, setLoadingFull] = useState(false);

  const toggleFull = useCallback(() => {
    setRenderFull((full) => {
      const fullp = !full;
      setLoadingFull(true);
      dataAgent
        .getPageSources(article.link)
        .then((full) => {
          var doc = new DOMParser().parseFromString(full.data, "text/html");
          var imgElements = doc.querySelectorAll("img");
          imgElements.forEach(function (img) {
            var newUrl = replaceImgUrl(img.src);
            img.setAttribute("src", newUrl);
          });
          var art = new Readability(doc).parse();
          if (art) {
            setPageContent(fullp ? art.content : originalContent ? originalContent : "");
          }
        })
        .finally(() => {
          setLoadingFull(false);
        });

      return fullp;
    });
  }, [setRenderFull, setPageContent, setLoadingFull, article]);

  function delegateContentClick(e: React.MouseEvent<HTMLElement>) {
    let elem = null;
    const i = e.nativeEvent.composedPath();

    for (let a = 0; a <= i.length - 1; a++) {
      const s = i[a] as HTMLElement;

      if ("A" === s.tagName) {
        elem = s;
        break;
      }
    }

    if (elem && elem.getAttribute("href")) {
      e.preventDefault();
      e.stopPropagation();

      const href = elem.getAttribute("href") || "";

      if (href && (href.indexOf("http://") >= 0 || href.indexOf("https://") >= 0 || href.indexOf("www.") >= 0)) {
        open(href);
      } else if (href.indexOf("#") === 0) {
        open(`${article.link}${href}`);
      }
    }
  }

  function renderMain() {
    const { isCommon, isYoutube, isPodcast } = validateFeed(article, medias || []);

    if (isYoutube) {
      return <YoutubeAdapter article={article} content={pageContent} medias={medias} />;
    } else if (isPodcast) {
      return <PodcastAdapter article={article} content={pageContent} medias={medias} />;
    } else {
      return (
        <div
          key={article.id}
          className={clsx("reading-content", "text-detail-paragraph")}
          // eslint-disable-next-line react/no-danger
          dangerouslySetInnerHTML={createMarkup(pageContent)}
        />
      );
    }
  }

  useEffect(() => {
    setPageContent("");

    article &&
      dataAgent
        .getArticleDetail(article.id, {
          signal: controller.signal,
        })
        .then((res) => {
          console.log("%c Line:102 🥓 res", "color:#33a5ff", res);
          const { data } = res;
          let content: React.SetStateAction<string> | undefined;

          if (data.content && data.description) {
            content = data.content.length > data.description.length ? data.content : data.description;
          } else {
            content = data.description || data.content || "";
          }

          content = content.replace(/<a[^>]+>/gi, (a: string) => {
            if (!/\starget\s*=/gi.test(a)) {
              return a.replace(/^<a\s/, '<a target="_blank"');
            }

            return a;
          });

          content = xss(content, {
            whiteList: {
              ...getDefaultWhiteList(),
              iframe: [],
              button: [],
            },
          });

          setPageContent(content);
          setOriginalContent((ori: any) => {
            if (ori === undefined) {
              return content;
            } else {
              return ori;
            }
          });

          try {
            if (data.media_object) {
              setMedias(JSON.parse(data.media_object));
              console.log(
                "%c Line:147 🌽 JSON.parse(data.media_object)",
                "color:#42b983",
                JSON.parse(data.media_object)
              );
            }
          } catch (e) {
            setMedias([]);
          }
        });

    return () => {
      controller.abort();
    };
  }, [article, setOriginalContent]);

  return (
    <div className="m-auto pt-1 pb-10 px-4 max-w-[calc(var(--reading-editable-line-width)_*_1px)]">
      <div className="pb-4 border-b border-border">
        <div className="mt-6 mb-5 text-4xl font-bold text-detail-headline">
          <a
            href={article.link}
            onClick={(event) => {
              event.preventDefault();

              window.open(article.link, "_blank");
            }}
          >
            {article.title}
          </a>
        </div>
        <div className={clsx(styles.meta)}>
          <span>
            <StarAndRead article={article} />
          </span>
          <TooltipBox content="Render full">
            <Icon
              className="w-7 h-7 !hover:text-[#fe9e2b]"
              onClick={(e: React.MouseEvent<HTMLElement>) => {
                e.stopPropagation();
                toggleFull();
              }}
            >
              {loadingFull ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : renderFull ? (
                <Scroll size={16} />
              ) : (
                <ScrollText size={16} />
              )}
            </Icon>
          </TooltipBox>
          <Separator orientation={"vertical"} className={"h-4 mx-2"} />
          <span className={styles.channelInfo}>
            <img src={replaceImgUrl(store.feed?.logo || ico)} alt="" className="rounded" />
            {article.feed_title}
          </span>

          <Separator orientation={"vertical"} className={"h-4 mx-2"} />

          {article.author && <span className={clsx(styles.author, "text-detail-paragraph")}>{article.author}</span>}
          <span className={clsx(styles.time, "text-detail-paragraph px-3")}>
            {pub_time ? Dayjs(new Date(pub_time)).format("YYYY-MM-DD HH:mm") : "Unknown Pub Date"}
          </span>
        </div>
      </div>
      <div className="m-auto pt-1 mt-6" onClick={delegateContentClick}>
        {article.image && (
          <div className="w-full my-4  text-center">
            <img src={article.image} alt="" className="bg-accent" />
          </div>
        )}
        {renderMain()}
      </div>
    </div>
  );
};
