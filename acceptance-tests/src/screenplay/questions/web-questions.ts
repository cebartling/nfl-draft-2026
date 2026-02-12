import type { Actor, Question } from '../actor.js';
import { BrowseTheWeb } from '../abilities/browse-the-web.js';

class PageHeadingQuestion implements Question<string> {
  async answeredBy(actor: Actor): Promise<string> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const heading = page.locator('h1').first();
    return (await heading.textContent()) ?? '';
  }
}

class CurrentUrlQuestion implements Question<string> {
  async answeredBy(actor: Actor): Promise<string> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    return page.url();
  }
}

export const PageHeading = {
  text(): Question<string> {
    return new PageHeadingQuestion();
  },
};

export const CurrentUrl = {
  value(): Question<string> {
    return new CurrentUrlQuestion();
  },
};
