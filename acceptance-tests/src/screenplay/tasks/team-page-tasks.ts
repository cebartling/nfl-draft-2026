import type { Actor, Task } from '../actor.js';
import { BrowseTheWeb } from '../abilities/browse-the-web.js';

class ExpandDivisionTask implements Task {
  constructor(private readonly divisionName: string) {}

  async performAs(actor: Actor): Promise<void> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const divisionButton = page.getByText(this.divisionName).first();
    await divisionButton.click();
  }
}

class ToggleTeamGroupingTask implements Task {
  constructor(private readonly mode: string) {}

  async performAs(actor: Actor): Promise<void> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.locator('button', { hasText: this.mode }).click();
  }
}

export const ExpandDivision = {
  named(divisionName: string): Task {
    return new ExpandDivisionTask(divisionName);
  },
};

export const ToggleTeamGrouping = {
  to(mode: string): Task {
    return new ToggleTeamGroupingTask(mode);
  },
};
