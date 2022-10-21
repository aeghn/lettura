import React, { useEffect, useState } from "react";
import { Modal, Table, Input } from "@douyinfe/semi-ui";
import { Channel } from "../../../db";
import * as dataAgent from "../../../helpers/dataAgent";
import styles from "./feedManage.module.scss";
import { TrashIcon } from "@heroicons/react/24/outline";

export const FeedManager = () => {
  const [list, setList] = useState<Channel[]>([]);
  const [searchText, setSearchText] = useState<string>("");

  const handleDeleteFeed = (channel: Channel) => {
    if (channel && channel.uuid) {
      Modal.confirm({
        title: "你确定要删除这个订阅吗？",
        content: channel.title,
        onOk: async() => {
          await dataAgent.deleteChannel(channel.uuid)
        },
      });
    }
  };

  const columns = [
    {
      title: "name",
      dataIndex: "title",
      render(text: string, record: Channel) {
        return (
          <div>
            <div>{text}</div>
            <div>{record.link}</div>
          </div>
        );
      },
    },
    {
      title: "Feed url",
      dataIndex: "feed_url",
      render(text: string, record: Channel): JSX.Element {
        return <div>{text}</div>;
      },
    },
    {
      title: "Action",
      dataIndex: "opt",
      width: 100,
      render(text: string, record: Channel): JSX.Element {
        return (
          <div>
            <span
              className={styles.delBtn}
              onClick={() => handleDeleteFeed(record)}
            >
              <TrashIcon className={"h4 w-4"} />
            </span>
          </div>
        );
      },
    },
  ];

  const handleSearch = (v: string) => {
    setSearchText(v);
    dataAgent.queryChannelWithKeywords(v).then((res) => {
      console.log("🚀 ~ file: index.tsx ~ line 67 ~ dataAgent.queryChannelWithKeywords ~ res", res)
      setList(res);
    })
  };

  const getList = async () => {
    const res = await dataAgent.getChannels() as Channel[];

    setList(res)
  }

  useEffect( () => {
    getList()
  }, []);

  return (
    <div>
      <div>
        <div>
          <Input
            placeholder="Search Feed"
            showClear
            value={searchText}
            onChange={handleSearch}
          />
        </div>
        <Table
          columns={columns}
          dataSource={list}
          pagination={false}
          size="small"
        />
      </div>
    </div>
  );
};
