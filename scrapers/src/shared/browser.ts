import { chromium, type Browser, type Page } from "playwright";

let browser: Browser | null = null;

/**
 * Launch a headless Chromium browser (reuses existing instance).
 */
export async function launchBrowser(): Promise<Browser> {
  if (!browser || !browser.isConnected()) {
    browser = await chromium.launch({ headless: true });
  }
  return browser;
}

/**
 * Fetch a URL with Playwright, wait for a selector, and return the rendered HTML.
 */
export async function fetchRenderedPage(
  url: string,
  waitSelector?: string,
  timeoutMs: number = 30000,
): Promise<string> {
  const b = await launchBrowser();
  const page: Page = await b.newPage();

  try {
    await page.goto(url, { waitUntil: "domcontentloaded", timeout: timeoutMs });

    if (waitSelector) {
      await page.waitForSelector(waitSelector, { timeout: timeoutMs });
    }

    // Give JS-rendered content a moment to populate after selector appears
    if (waitSelector) {
      await page.waitForTimeout(2000);
    }

    return await page.content();
  } finally {
    await page.close();
  }
}

/**
 * Close the shared browser instance.
 */
export async function closeBrowser(): Promise<void> {
  if (browser && browser.isConnected()) {
    await browser.close();
    browser = null;
  }
}
