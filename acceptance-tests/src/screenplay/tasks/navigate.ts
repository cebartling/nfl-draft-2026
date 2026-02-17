import type { Actor, Task } from '../actor.js';
import { BrowseTheWeb } from '../abilities/browse-the-web.js';

class NavigateTo implements Task {
  constructor(private readonly path: string) {}

  async performAs(actor: Actor): Promise<void> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    await page.goto(this.path);
  }
}

export const Navigate = {
  to(path: string): Task {
    return new NavigateTo(path);
  },
  toHomePage(): Task {
    return new NavigateTo('/');
  },
  toTeams(): Task {
    return new NavigateTo('/teams');
  },
  toPlayers(): Task {
    return new NavigateTo('/players');
  },
  toDrafts(): Task {
    return new NavigateTo('/drafts');
  },
  toProspects(): Task {
    return new NavigateTo('/prospects');
  },
};
