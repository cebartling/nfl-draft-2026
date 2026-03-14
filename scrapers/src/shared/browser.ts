import { chromium, type Browser } from "playwright";

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
 * Close the shared browser instance.
 */
export async function closeBrowser(): Promise<void> {
  if (browser && browser.isConnected()) {
    await browser.close();
    browser = null;
  }
}
