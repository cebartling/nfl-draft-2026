import type { Page } from '@playwright/test';
import type { Ability } from '../actor.js';

export class BrowseTheWeb implements Ability {
  private constructor(private readonly page: Page) {}

  static using(page: Page): BrowseTheWeb {
    return new BrowseTheWeb(page);
  }

  getPage(): Page {
    return this.page;
  }
}
