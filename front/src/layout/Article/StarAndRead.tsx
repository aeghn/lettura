import { TooltipBox } from "@/components/TooltipBox";
import { Icon } from "@/components/Icon";
import { CheckCircle2, Circle, Star } from "lucide-react";
import { ArticleReadStatus, ArticleStarStatus } from "@/typing";
import React, { useEffect, useState } from "react";
import { ArticleResItem } from "@/db";
import * as dataAgent from "@/helpers/dataAgent";

export interface StarAndReadProps {
  article: ArticleResItem;
}

export function StarAndRead(props: StarAndReadProps) {
  const { article } = props;
  const [isRead, setIsRead] = useState<boolean>();
  const [isStarred, setIsStarred] = useState<boolean>();

  function toggleReadStatus() {
    let newStatus: boolean = false;

    if (isRead === false) {
      newStatus = true;
    } else {
      newStatus = false;
    }

    dataAgent.updateArticleReadStatus(article.id, newStatus).then(() => {
      article.is_read = newStatus;
      setIsRead(newStatus);
    });
  }

  function toggleStarStatus() {
    let newIsStarred: boolean = true;

    if (isStarred === false) {
      newIsStarred = true;
    } else {
      newIsStarred = false;
    }

    dataAgent.updateArticleStarStatus(article.id, newIsStarred).then(() => {
      article.is_starred = newIsStarred;
      setIsStarred(newIsStarred);
    });
  }

  useEffect(() => {
    setIsRead(article.is_read);
  }, [article.is_read]);

  useEffect(() => {
    setIsStarred(article.is_starred);
  }, [article.is_starred]);

  return (
    <>
      {article.is_starred === false && (
        <TooltipBox content="Star it">
          <Icon
            className="w-7 h-7"
            onClick={(e: React.MouseEvent<HTMLElement>) => {
              e.stopPropagation();
              toggleStarStatus();
            }}
          >
            <Star size={16} />
          </Icon>
        </TooltipBox>
      )}
      {article.is_starred === true && (
        <TooltipBox content="Unstar it">
          <Icon
            className="w-7 h-7 !text-[#fe9e2b] !hover:text-[#fe9e2b]"
            onClick={(e: React.MouseEvent<HTMLElement>) => {
              e.stopPropagation();
              toggleStarStatus();
            }}
          >
            <Star size={16} fill={"currentColor"} />
          </Icon>
        </TooltipBox>
      )}
      {article.is_read === false && (
        <TooltipBox content="Mark as read">
          <Icon
            className="w-7 h-7"
            onClick={(e: React.MouseEvent<HTMLElement>) => {
              e.stopPropagation();
              toggleReadStatus();
            }}
          >
            <Circle size={16} />
          </Icon>
        </TooltipBox>
      )}
      {article.is_read === true && (
        <TooltipBox content="Mark as unread">
          <Icon
            className="w-7 h-7"
            onClick={(e: React.MouseEvent<HTMLElement>) => {
              e.stopPropagation();
              toggleReadStatus();
            }}
          >
            <CheckCircle2 size={16} />
          </Icon>
        </TooltipBox>
      )}
    </>
  );
}
