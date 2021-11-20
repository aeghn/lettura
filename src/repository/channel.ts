import { EntityRepository, Repository, getConnection } from 'typeorm';
import { ArticleEntity } from '../entity/article';
import { ChannelEntity } from '../entity/channel';
import { ArticleReadStatus } from '../infra/constants/status';
import { Channel } from '../infra/types';

type Response = {
  status: 'success' | 'fail';
  message: string;
  data?: any;
};

@EntityRepository(ChannelEntity)
export class ChannelRepository extends Repository<Channel> {
  /**
   * 获取所有订阅和未读的文章数量
   */
  async getAll() {
    const list: ChannelEntity[] = await this.find({});
    const channelIdList = list.reduce((acu, cur: ChannelEntity) => {
      acu.push(cur.id);
      return acu;
    }, [] as string[]);

    const counterMap = new Map();
    const counter: {
      channelId: string;
      total: number;
    }[] = await getConnection()
      .getRepository(ArticleEntity)
      .createQueryBuilder('article')
      .groupBy('article.channelId')
      .where('channelId IN (:...channelIdList)', { channelIdList })
      .andWhere('article.hasRead = :hasRead', {
        hasRead: ArticleReadStatus.unRead,
      })
      .select(['article.channelId as channelId', 'count(article.id) as total'])
      .execute();

    counter.forEach((item) => {
      counterMap.set(item.channelId, item.total);
    });

    return list.map(
      (item: ChannelEntity): Channel => {
        return { ...item, articleCount: counterMap.get(item.id) || 0 };
      }
    );
  }

  async addOne(feed: Channel): Promise<ChannelEntity> {
    const channel = await this.findOne({
      where: {
        feedUrl: feed.feedUrl,
      },
    });

    if (channel) {
      throw new Error(`您已经订阅了该频道：${channel.title}`);
    }

    feed.updateDate = new Date().toString();
    feed.lastSyncDate = new Date().toString();

    return this.save(feed);
  }

  async getList(): Promise<ChannelEntity[]> {
    return this.find({});
  }

  async getOneById(id: string): Promise<ChannelEntity> {
    const channel = await this.find({
      where: {
        id,
      },
    });

    return channel[0];
  }

  async getOneByUrl(url: string): Promise<ChannelEntity> {
    const channel = await this.find({
      where: {
        feedUrl: url,
      },
    });

    return channel[0];
  }

  async updateLastSyncDate(id: string): Promise<any> {
    const channel = await this.findOne({
      where: {
        id,
      },
    });

    if (channel) {
      channel.updateDate = new Date().toString();
      channel.lastSyncDate = new Date().toString();

      return this.save(channel);
    }

    return null;
  }

  /**
   * 添加 Channel
   * @param {RSSFeed} channel 解析出来的内容
   * @param {RSSFeedItem[]} articles
   */
  async subscribeChannel(channel: Channel): Promise<ChannelEntity | string> {
    try {
      const result = this.addOne(channel);

      return result;
    } catch (err) {
      console.error(err.message);
    }

    return '';
  }

  /**
   * 取消订阅
   * @param channelId 订阅id
   */
  async cancelSubscribe(
    channelId: string
  ): Promise<
    Response | { data: { articleChanges: number; channelChanges: number } }
  > {
    const channel = await this.getOneById(channelId);

    if (!channel) {
      return {
        status: 'fail',
        message: '未订阅该频道',
      };
    }

    const { raw: articleRaw } = await getConnection()
      .getRepository(ArticleEntity)
      .createQueryBuilder('article')
      .delete()
      .where('channelId=:channelId', { channelId })
      .execute();

    const { raw: channelRaw } = await this.createQueryBuilder('channel')
      .delete()
      .where('id=:channelId', { channelId })
      .execute();

    return {
      status: 'success',
      message: '',
      data: {
        articleChanges: articleRaw.changes || 0,
        channelChanges: channelRaw.changes || 0,
      },
    };
  }

  /**
   * 更新频道信息
   * @param channelId
   * @param body Partial<ChannelEntity>
   */
  async updateInfo(
    channelId: string,
    body: Partial<ChannelEntity>
  ): Promise<any> {
    const channel = await this.getOneById(channelId);

    if (!channel) {
      return {
        status: 'fail',
        message: '未订阅该频道',
      };
    }

    return this.update(channelId, {
      title: body.title,
      description: body.description,
      feedUrl: body.feedUrl,
      link: body.link,
      favicon: body.favicon,
    });
  }
}
