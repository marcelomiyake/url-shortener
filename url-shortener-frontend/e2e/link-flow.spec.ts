import { expect, test } from '@playwright/test';

test('creates and copies a short link, redirects, and reports an unknown code', async ({ page, context, request }) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  await page.goto('/');
  const destination = `https://example.com/e2e-${Date.now()}`;
  await page.getByLabel('Destination URL').fill(destination);
  await page.getByRole('button', { name: 'Create short link' }).click();

  const link = page.getByTestId('short-link');
  await expect(link).toBeVisible();
  const shortUrl = new URL(await link.getAttribute('href') ?? '');
  expect(shortUrl.origin).toBe(new URL(page.url()).origin);
  expect(shortUrl.pathname).toMatch(/^\/[0-9A-Za-z]{12,43}$/);

  await page.getByRole('button', { name: 'Copy link' }).click();
  await expect(page.getByRole('status')).toHaveText('Short link copied.');
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(shortUrl.href);

  const redirect = await request.get(shortUrl.href, { maxRedirects: 0 });
  expect(redirect.status()).toBe(301);
  expect(redirect.headers().location).toBe(destination);
  expect(redirect.headers()['cache-control']).toBe('public, max-age=3600');

  const unknown = await request.get(new URL('/ZZZZZZZZZZZZ', page.url()).href, { maxRedirects: 0 });
  expect(unknown.status()).toBe(404);
});
