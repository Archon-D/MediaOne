export interface Subscription { id: string; platform: string; url: string; title: string; lastSyncedAt: string | null; lastError: string | null }
export interface MediaItem { id: string; sourceId: string; externalId: string; title: string; url: string | null; publishedAt: string | null; fetchedAt: string; isRead: boolean }
export interface FeedFilter { sourceId?: string; unreadOnly: boolean; limit?: number; offset?: number }
export interface ConnectorInfo { platform: string; implemented: boolean; manualSource: boolean; subscriptionImport: boolean; authenticatedAccess: boolean }
export interface Account { id: string; platform: string; displayName: string; status: string }
export interface SyncReport { sourceId: string; itemCount: number; error: string | null }
