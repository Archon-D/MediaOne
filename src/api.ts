import { invoke, isTauri } from '@tauri-apps/api/core';
import type { Account, ConnectorInfo, FeedFilter, MediaItem, Subscription, SyncReport } from './types';
function local<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) return Promise.reject(new Error('Open MediaOne in the native app to access your local feeds.'));
  return invoke<T>(command, args);
}
export const api = {
  feed: (filter: FeedFilter) => local<MediaItem[]>('get_feed', { filter }),
  subscriptions: () => local<Subscription[]>('list_subscriptions'),
  add: (url: string) => local<Subscription>('add_subscription', { url }),
  remove: (id: string) => local<void>('remove_subscription', { id }),
  read: (id: string, isRead: boolean) => local<void>('set_read_state', { id, isRead }),
  refresh: () => local<SyncReport[]>('refresh_feeds'),
  connectors: () => local<ConnectorInfo[]>('list_connectors'),
  accounts: () => local<Account[]>('list_accounts'),
};
