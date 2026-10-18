import { cleanup, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import App from './App';
import { api } from './api';
vi.mock('./api', () => ({ api: { feed: vi.fn(), subscriptions: vi.fn(), connectors: vi.fn(), accounts: vi.fn(), add: vi.fn(), remove: vi.fn(), read: vi.fn(), refresh: vi.fn() } }));
const source = { id: 's', platform: 'rss', url: 'https://example.com/feed', title: 'Example', lastSyncedAt: null, lastError: null };
beforeEach(() => {
  vi.mocked(api.subscriptions).mockResolvedValue([]); vi.mocked(api.feed).mockResolvedValue([]);
  vi.mocked(api.connectors).mockResolvedValue([]); vi.mocked(api.accounts).mockResolvedValue([]);
  vi.mocked(api.add).mockResolvedValue(source); vi.mocked(api.read).mockResolvedValue();
  vi.mocked(api.refresh).mockResolvedValue([]);
});
afterEach(() => { cleanup(); vi.resetAllMocks(); });
describe('local reading workflow', () => {
  it('adds a subscription without pretending a fetch has succeeded', async () => {
    const user = userEvent.setup(); render(<App />);
    await user.click(screen.getByRole('button', { name: 'Subscriptions' }));
    await user.type(screen.getByLabelText('RSS / Atom feed URL'), source.url);
    await user.click(screen.getByRole('button', { name: 'Add source' }));
    await waitFor(() => expect(api.add).toHaveBeenCalledWith(source.url));
    expect(await screen.findByRole('status')).toHaveProperty('textContent', 'Subscription saved. Refresh feeds to fetch articles.');
  });
  it('marks articles read through IPC and requests unread filtering', async () => {
    vi.mocked(api.subscriptions).mockResolvedValue([source]);
    vi.mocked(api.feed).mockResolvedValue([{ id: 'i', sourceId: 's', externalId: 'e', title: '<script>plain title</script>', url: 'https://example.com/article', publishedAt: null, fetchedAt: '2026-10-18T00:00:00Z', isRead: false }]);
    const user = userEvent.setup(); render(<App />);
    await user.click(await screen.findByRole('button', { name: 'Mark read' }));
    await waitFor(() => expect(api.read).toHaveBeenCalledWith('i', true));
    await waitFor(() => expect((screen.getByRole('button', { name: 'Mark read' }) as HTMLButtonElement).disabled).toBe(false));
    await user.click(screen.getByRole('checkbox', { name: 'Unread only' }));
    await waitFor(() => expect(api.feed).toHaveBeenLastCalledWith(expect.objectContaining({ unreadOnly: true })));
    expect(document.querySelector('article script')).toBeNull();
  });
  it('surfaces failures while keeping the cached feed', async () => {
    vi.mocked(api.refresh).mockRejectedValue(new Error('Offline'));
    const user = userEvent.setup(); render(<App />);
    await user.click(screen.getByRole('button', { name: 'Refresh feeds' }));
    expect((await screen.findByRole('alert')).textContent).toContain('Offline');
  });
});
