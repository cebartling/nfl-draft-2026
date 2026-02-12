import type { Actor, Task } from '../actor.js';
import { BrowseTheWeb } from '../abilities/browse-the-web.js';

class SearchForPlayerTask implements Task {
  constructor(private readonly query: string) {}

  async performAs(actor: Actor): Promise<void> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const searchInput = page.locator('#search');
    await searchInput.clear();
    await searchInput.fill(this.query);
  }
}

class FilterByPositionGroupTask implements Task {
  constructor(private readonly group: string) {}

  async performAs(actor: Actor): Promise<void> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const groupSelect = page.locator('#group');
    await groupSelect.selectOption({ label: this.group });
  }
}

export const SearchForPlayer = {
  named(query: string): Task {
    return new SearchForPlayerTask(query);
  },
};

export const FilterByPositionGroup = {
  named(group: string): Task {
    return new FilterByPositionGroupTask(group);
  },
};
