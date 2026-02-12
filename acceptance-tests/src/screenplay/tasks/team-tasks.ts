import type { Actor, Task } from '../actor.js';
import { BrowseTheWeb } from '../abilities/browse-the-web.js';

class ViewTeamDetailTask implements Task {
  constructor(private readonly teamName: string) {}

  async performAs(actor: Actor): Promise<void> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.getByText(this.teamName, { exact: false }).first().click();
    await page.waitForURL(/\/teams\/[0-9a-f-]+/);
  }
}

export const ViewTeamDetail = {
  forTeam(teamName: string): Task {
    return new ViewTeamDetailTask(teamName);
  },
};
