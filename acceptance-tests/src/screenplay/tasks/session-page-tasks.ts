import type { Actor, Task } from '../actor.js';
import { BrowseTheWeb } from '../abilities/browse-the-web.js';

/**
 * Waits for the WebSocket connection indicator to show "Connected" status.
 */
class WaitForWebSocketConnectionTask implements Task {
  private timeoutMs: number = 15_000;

  withTimeout(ms: number): this {
    this.timeoutMs = ms;
    return this;
  }

  async performAs(actor: Actor): Promise<void> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page
      .locator('[data-testid="ws-connection-status"]:has-text("Connected")')
      .waitFor({ state: 'visible', timeout: this.timeoutMs });
  }
}

export const WaitForWebSocketConnection = {
  onPage(): WaitForWebSocketConnectionTask {
    return new WaitForWebSocketConnectionTask();
  },
};

/**
 * Waits for a specific pick card to show a player name, indicating the pick was made.
 */
class WaitForPickCardUpdateTask implements Task {
  private timeoutMs: number = 30_000;

  constructor(private readonly pickNumber: number) {}

  withTimeout(ms: number): this {
    this.timeoutMs = ms;
    return this;
  }

  async performAs(actor: Actor): Promise<void> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    // Wait for the pick card to contain a non-empty player name
    await page
      .locator(
        `[data-pick="${this.pickNumber}"] [data-testid="pick-player-name"]`,
      )
      .waitFor({ state: 'visible', timeout: this.timeoutMs });
  }
}

export const WaitForPickCardUpdate = {
  forPick(n: number): WaitForPickCardUpdateTask {
    return new WaitForPickCardUpdateTask(n);
  },
};

/**
 * Waits until a specified number of pick cards have player names assigned.
 */
class WaitForAllPicksFilledTask implements Task {
  private timeoutMs: number = 60_000;

  constructor(private readonly expectedCount: number) {}

  withTimeout(ms: number): this {
    this.timeoutMs = ms;
    return this;
  }

  async performAs(actor: Actor): Promise<void> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    // Poll until the expected number of filled pick cards appears
    await page.waitForFunction(
      (expected: number) => {
        const cards = document.querySelectorAll(
          '[data-pick] [data-testid="pick-player-name"]',
        );
        let filled = 0;
        cards.forEach((card) => {
          if (card.textContent && card.textContent.trim() !== '') {
            filled++;
          }
        });
        return filled >= expected;
      },
      this.expectedCount,
      { timeout: this.timeoutMs },
    );
  }
}

export const WaitForAllPicksFilled = {
  expecting(n: number): WaitForAllPicksFilledTask {
    return new WaitForAllPicksFilledTask(n);
  },
};
