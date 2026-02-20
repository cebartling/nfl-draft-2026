import type { Actor, Question } from '../actor.js';
import { BrowseTheWeb } from '../abilities/browse-the-web.js';

/**
 * Whether a specific pick card shows a player name (indicating the pick has been made).
 */
class PickCardHasPlayerQuestion implements Question<boolean> {
  constructor(private readonly pickNumber: number) {}

  async answeredBy(actor: Actor): Promise<boolean> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const card = page.locator(`[data-pick="${this.pickNumber}"]`);
    // A pick card with a player shows player name text content beyond just team/pick info
    const playerName = card.locator('[data-testid="pick-player-name"]');
    return (await playerName.count()) > 0 && (await playerName.textContent()) !== '';
  }
}

export const PickCardHasPlayer = {
  forPick(n: number): Question<boolean> {
    return new PickCardHasPlayerQuestion(n);
  },
};

/**
 * Count of pick cards on the draft board that have a player name assigned.
 */
class DraftBoardFilledPickCountQuestion implements Question<number> {
  async answeredBy(actor: Actor): Promise<number> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const filledCards = page.locator('[data-pick] [data-testid="pick-player-name"]');
    const count = await filledCards.count();
    // Only count cards where the player name is non-empty
    let filled = 0;
    for (let i = 0; i < count; i++) {
      const text = await filledCards.nth(i).textContent();
      if (text && text.trim() !== '') {
        filled++;
      }
    }
    return filled;
  }
}

export const DraftBoardFilledPickCount = {
  onPage(): Question<number> {
    return new DraftBoardFilledPickCountQuestion();
  },
};

/**
 * The data-pick value of the currently highlighted pick card, or null if none.
 */
class CurrentPickHighlightQuestion implements Question<number | null> {
  async answeredBy(actor: Actor): Promise<number | null> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const highlighted = page.locator('[data-pick][data-current="true"]');
    const count = await highlighted.count();
    if (count === 0) return null;
    const value = await highlighted.first().getAttribute('data-pick');
    return value ? parseInt(value, 10) : null;
  }
}

export const CurrentPickHighlight = {
  onPage(): Question<number | null> {
    return new CurrentPickHighlightQuestion();
  },
};

/**
 * Count of pick notification items in the activity feed.
 */
class PickNotificationCountQuestion implements Question<number> {
  async answeredBy(actor: Actor): Promise<number> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    return page.locator('[data-testid="pick-notification"]').count();
  }
}

export const PickNotificationCount = {
  onPage(): Question<number> {
    return new PickNotificationCountQuestion();
  },
};

/**
 * Text content of the session status element.
 */
class SessionStatusTextQuestion implements Question<string> {
  async answeredBy(actor: Actor): Promise<string> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const status = page.locator('[data-testid="session-status"]');
    return (await status.textContent()) ?? '';
  }
}

export const SessionStatusText = {
  onPage(): Question<string> {
    return new SessionStatusTextQuestion();
  },
};

/**
 * Whether the WebSocket connection indicator shows "Connected".
 */
class WebSocketIsConnectedQuestion implements Question<boolean> {
  async answeredBy(actor: Actor): Promise<boolean> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const indicator = page.locator('[data-testid="ws-connection-status"]');
    const count = await indicator.count();
    if (count === 0) return false;
    const text = await indicator.textContent();
    return text?.toLowerCase().includes('connected') ?? false;
  }
}

export const WebSocketIsConnected = {
  onPage(): Question<boolean> {
    return new WebSocketIsConnectedQuestion();
  },
};
