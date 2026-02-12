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

class DisplayedCountQuestion implements Question<number> {
  constructor(private readonly selector: string) {}

  async answeredBy(actor: Actor): Promise<number> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    return page.locator(this.selector).count();
  }
}

class ElementTextQuestion implements Question<string> {
  constructor(private readonly selector: string) {}

  async answeredBy(actor: Actor): Promise<string> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const element = page.locator(this.selector).first();
    return (await element.textContent()) ?? '';
  }
}

class IsVisibleQuestion implements Question<boolean> {
  constructor(private readonly selector: string) {}

  async answeredBy(actor: Actor): Promise<boolean> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    return page.locator(this.selector).first().isVisible();
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

export const DisplayedCount = {
  of(selector: string): Question<number> {
    return new DisplayedCountQuestion(selector);
  },
};

export const ElementText = {
  of(selector: string): Question<string> {
    return new ElementTextQuestion(selector);
  },
};

export const IsVisible = {
  element(selector: string): Question<boolean> {
    return new IsVisibleQuestion(selector);
  },
};
