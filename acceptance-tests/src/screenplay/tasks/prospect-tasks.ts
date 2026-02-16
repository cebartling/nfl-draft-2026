import type { Actor, Task } from '../actor.js';
import { BrowseTheWeb } from '../abilities/browse-the-web.js';

class SelectPositionTask implements Task {
  constructor(private readonly position: string) {}

  async performAs(actor: Actor): Promise<void> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const positionSelect = page.locator('#position');
    await positionSelect.selectOption(this.position);
  }
}

class ClearAllFiltersTask implements Task {
  async performAs(actor: Actor): Promise<void> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.getByText('Clear all').click();
  }
}

export const SelectPosition = {
  named(position: string): Task {
    return new SelectPositionTask(position);
  },
};

export const ClearAllFilters = {
  now(): Task {
    return new ClearAllFiltersTask();
  },
};
