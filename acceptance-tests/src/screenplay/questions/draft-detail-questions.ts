import type { Actor, Question } from '../actor.js';
import { BrowseTheWeb } from '../abilities/browse-the-web.js';

class TeamSelectorVisibleQuestion implements Question<boolean> {
  async answeredBy(actor: Actor): Promise<boolean> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const heading = page.getByRole('heading', { name: /select teams to control/i });
    return heading.isVisible();
  }
}

class AutoPickButtonVisibleQuestion implements Question<boolean> {
  async answeredBy(actor: Actor): Promise<boolean> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const button = page.getByTestId('auto-pick-all');
    return button.isVisible();
  }
}

class StartButtonTextQuestion implements Question<string> {
  async answeredBy(actor: Actor): Promise<string> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const button = page.getByTestId('start-with-teams');
    return (await button.textContent())?.trim() ?? '';
  }
}

class SelectedTeamCountQuestion implements Question<number> {
  async answeredBy(actor: Actor): Promise<number> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    // The TeamSelector component shows "{N} team(s) selected" text
    const countText = page.locator('text=/\\d+ teams? selected/');
    const text = await countText.textContent();
    const match = text?.match(/(\d+) teams? selected/);
    return match ? parseInt(match[1], 10) : 0;
  }
}

class CancelButtonVisibleQuestion implements Question<boolean> {
  async answeredBy(actor: Actor): Promise<boolean> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const button = page.getByTestId('cancel-draft');
    return button.isVisible();
  }
}

export const TeamSelectorVisible = {
  onPage(): Question<boolean> {
    return new TeamSelectorVisibleQuestion();
  },
};

export const AutoPickButtonVisible = {
  onPage(): Question<boolean> {
    return new AutoPickButtonVisibleQuestion();
  },
};

export const StartButtonText = {
  onPage(): Question<string> {
    return new StartButtonTextQuestion();
  },
};

export const SelectedTeamCount = {
  onPage(): Question<number> {
    return new SelectedTeamCountQuestion();
  },
};

export const CancelButtonVisible = {
  onPage(): Question<boolean> {
    return new CancelButtonVisibleQuestion();
  },
};

class DraftProgressVisibleQuestion implements Question<boolean> {
  async answeredBy(actor: Actor): Promise<boolean> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const section = page.getByTestId('draft-progress');
    return section.isVisible();
  }
}

class DraftBoardVisibleQuestion implements Question<boolean> {
  async answeredBy(actor: Actor): Promise<boolean> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const section = page.getByTestId('draft-board');
    return section.isVisible();
  }
}

class DraftStatisticsVisibleQuestion implements Question<boolean> {
  async answeredBy(actor: Actor): Promise<boolean> {
    const page = actor.abilityTo(BrowseTheWeb).getPage();
    const section = page.getByTestId('draft-statistics');
    return section.isVisible();
  }
}

export const DraftProgressVisible = {
  onPage(): Question<boolean> {
    return new DraftProgressVisibleQuestion();
  },
};

export const DraftBoardVisible = {
  onPage(): Question<boolean> {
    return new DraftBoardVisibleQuestion();
  },
};

export const DraftStatisticsVisible = {
  onPage(): Question<boolean> {
    return new DraftStatisticsVisibleQuestion();
  },
};
