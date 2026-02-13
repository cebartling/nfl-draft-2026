import type { Actor, Task } from '../actor.js';
import { BrowseTheWeb } from '../abilities/browse-the-web.js';

class SelectTeamByNameTask implements Task {
  constructor(private readonly teamName: string) {}

  async performAs(actor: Actor): Promise<void> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();

    // Expand all divisions to make the team visible
    const accordionButtons = page.locator(
      'button.w-full.flex.items-center.justify-between',
    );
    const count = await accordionButtons.count();
    for (let i = 0; i < count; i++) {
      const btn = accordionButtons.nth(i);
      // Check if the next sibling (expanded content) is not visible
      const svg = btn.locator('svg');
      const svgClass = await svg.getAttribute('class');
      if (svgClass && !svgClass.includes('rotate-180')) {
        await btn.click();
      }
    }

    // Now click the team card that matches the name
    const teamButton = page.locator(`button:has-text("${this.teamName}")`).first();
    await teamButton.click();
  }
}

class ClickAutoPickAllTeamsTask implements Task {
  async performAs(actor: Actor): Promise<void> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.getByTestId('auto-pick-all').click();
  }
}

class ClickStartWithTeamsTask implements Task {
  async performAs(actor: Actor): Promise<void> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.getByTestId('start-with-teams').click();
  }
}

class ClickCancelTask implements Task {
  async performAs(actor: Actor): Promise<void> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.getByTestId('cancel-draft').click();
  }
}

export const SelectTeam = {
  byName(name: string): Task {
    return new SelectTeamByNameTask(name);
  },
};

export const ClickAutoPickAllTeams = {
  now(): Task {
    return new ClickAutoPickAllTeamsTask();
  },
};

export const ClickStartWithTeams = {
  now(): Task {
    return new ClickStartWithTeamsTask();
  },
};

export const ClickCancel = {
  now(): Task {
    return new ClickCancelTask();
  },
};
