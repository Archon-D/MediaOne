import { useCallback, useEffect, useRef, useState } from 'react';
import { api } from './api';
import type { Account, ConnectorInfo, MediaItem, Subscription } from './types';

const PAGE = 50;
export default function App() {
  const [view, setView] = useState<'feed' | 'subscriptions' | 'accounts'>('feed');
  const [sources, setSources] = useState<Subscription[]>([]);
  const [items, setItems] = useState<MediaItem[]>([]);
  const [connectors, setConnectors] = useState<ConnectorInfo[]>([]);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [sourceId, setSourceId] = useState('');
  const [unreadOnly, setUnreadOnly] = useState(false);
  const [limit, setLimit] = useState(PAGE);
  const [url, setUrl] = useState('');
  const [error, setError] = useState('');
  const [notice, setNotice] = useState('');
  const [busy, setBusy] = useState(false);
  const operation = useRef(false);
  const generation = useRef(0);
  const mounted = useRef(true);
  const load = useCallback(async () => {
    const current = ++generation.current;
    const [nextSources, nextItems, nextConnectors, nextAccounts] = await Promise.all([
      api.subscriptions(), api.feed({ sourceId: sourceId || undefined, unreadOnly, limit }), api.connectors(), api.accounts(),
    ]);
    if (!mounted.current || generation.current !== current) return;
    setSources(nextSources); setItems(nextItems); setConnectors(nextConnectors); setAccounts(nextAccounts);
  }, [sourceId, unreadOnly, limit]);
  useEffect(() => {
    mounted.current = true;
    void load().catch(e => { if (mounted.current) setError(String(e)); });
    return () => { mounted.current = false; generation.current++; };
  }, [load]);
  const perform = useCallback(async (action: () => Promise<void>) => {
    if (operation.current) return;
    operation.current = true; setBusy(true); setError(''); setNotice('');
    try { await action(); await load(); } catch (e) { if (mounted.current) setError(String(e)); }
    finally { operation.current = false; if (mounted.current) setBusy(false); }
  }, [load]);
  const refresh = useCallback(() => perform(async () => {
    const reports = await api.refresh();
    const failed = reports.filter(r => r.error);
    setNotice(reports.length === 0 ? 'Add a subscription first.' : `${reports.length - failed.length} sources refreshed; ${failed.length} failed. Cached articles remain available.`);
  }), [perform]);
  useEffect(() => {
    const timer = window.setInterval(() => { if (document.visibilityState === 'visible') void refresh(); }, 5 * 60_000);
    return () => window.clearInterval(timer);
  }, [refresh]);
  return <div className="shell">
    <aside><div className="brand"><span>m₁</span> MediaOne</div><p className="tagline">Your subscriptions. Your pace.</p>
      <nav aria-label="Main navigation">{(['feed', 'subscriptions', 'accounts'] as const).map(v => <button key={v} aria-current={view === v ? 'page' : undefined} onClick={() => setView(v)}>{v === 'feed' ? 'Reading room' : v === 'subscriptions' ? 'Subscriptions' : 'Accounts'}</button>)}</nav>
      <div className="local"><span className="dot"/> Stored on this device<br/><small>No central server · no recommendations</small></div>
    </aside>
    <main><header><div><p className="eyebrow">A QUIETER CORNER OF THE INTERNET</p><h1>{view === 'feed' ? 'Reading room' : view === 'subscriptions' ? 'Your sources' : 'Connected accounts'}</h1></div><button className="primary" disabled={busy} onClick={() => void refresh()}>{busy ? 'Working…' : 'Refresh feeds'}</button></header>
      {error && <div role="alert" className="message error">{error}</div>}{notice && <div role="status" className="message">{notice}</div>}
      {view === 'feed' && <><div className="toolbar"><label><input type="checkbox" checked={unreadOnly} disabled={busy} onChange={e => { setUnreadOnly(e.target.checked); setLimit(PAGE); }}/> Unread only</label><label>Source <select aria-label="Source" disabled={busy} value={sourceId} onChange={e => { setSourceId(e.target.value); setLimit(PAGE); }}><option value="">All subscriptions</option>{sources.map(s => <option key={s.id} value={s.id}>{s.title}</option>)}</select></label><span>Newest first</span></div>
        {sources.some(s => s.lastError) && <p className="message">Some sources could not refresh. See Subscriptions for details.</p>}
        {items.length === 0 ? <div className="empty"><h2>{sources.length ? 'You’re all caught up.' : 'Build a feed worth opening.'}</h2><p>{sources.length ? 'Refresh your sources or change your filters.' : 'Add an RSS or Atom URL to begin your own reading room.'}</p><button onClick={() => setView('subscriptions')}>Manage subscriptions</button></div> : <section aria-label="Feed" className="feed">{items.map(item => <article key={item.id} className={item.isRead ? 'read' : ''}><div className="item-meta"><span>{sources.find(s => s.id === item.sourceId)?.title ?? 'RSS'}</span><time>{item.publishedAt ? new Date(item.publishedAt).toLocaleString() : 'Publication date unavailable'}</time></div><h2>{item.title}</h2><div className="item-actions">{item.url && /^https?:\/\//i.test(item.url) && <button disabled={busy} onClick={() => void perform(() => api.open(item.id))}>Open original ↗</button>}<button disabled={busy} onClick={() => void perform(() => api.read(item.id, !item.isRead))}>{item.isRead ? 'Mark unread' : 'Mark read'}</button></div></article>)}</section>}
        {items.length >= limit && limit < 200 && <button disabled={busy} onClick={() => setLimit(n => Math.min(n + PAGE, 200))}>Load more</button>}
        {items.length === 200 && <p>Showing the newest 200 articles. Filter by source to narrow the list.</p>}</>}
      {view === 'subscriptions' && <><p className="intro">Follow sources you choose. RSS and Atom work without an account.</p><form onSubmit={e => { e.preventDefault(); void perform(async () => { await api.add(url); setUrl(''); setNotice('Subscription saved. Refresh feeds to fetch articles.'); }); }}><label htmlFor="feed-url">RSS / Atom feed URL</label><div className="form-row"><input id="feed-url" type="url" required placeholder="https://example.com/feed.xml" value={url} disabled={busy} onChange={e => setUrl(e.target.value)}/><button className="primary" disabled={busy || !url.trim()}>Add source</button></div></form>
        <section aria-label="Subscriptions">{sources.map(source => <article key={source.id}><h2>{source.title}</h2><p className="source-url">{source.url}</p><p className="muted">{source.lastSyncedAt ? `Last refreshed ${new Date(source.lastSyncedAt).toLocaleString()}` : 'Not fetched yet'}</p>{source.lastError && <p className="error">{source.lastError}</p>}<button disabled={busy} onClick={() => void perform(async () => { await api.remove(source.id); if (sourceId === source.id) setSourceId(''); })}>Remove subscription</button></article>)}</section></>}
      {view === 'accounts' && <><p className="intro">RSS needs no login. Platform login and secure session storage are planned; connection controls will appear when each adapter is ready.</p><section aria-label="Platform connectors" className="connector-grid">{connectors.map(c => <article key={c.platform}><h2>{c.platform === 'rss' ? 'RSS / Atom' : c.platform}</h2><p>{c.implemented ? 'Available · public feeds' : 'Planned · not connected'}</p></article>)}</section>{accounts.map(a => <p key={a.id}>{a.displayName} · {a.platform} · {a.status}</p>)}</>}
    </main>
  </div>;
}
