/**
 * Screenplay pattern core: Actor, Ability, Task, and Question interfaces.
 *
 * Tests read like user stories:
 *   await actor.attemptsTo(Navigate.to('/drafts/new'), CreateDraft.named('My Draft'));
 *   const status = await actor.asks(DraftStatus.inDatabaseFor(draftId));
 */

// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface Ability {}

export interface Task {
  performAs(actor: Actor): Promise<void>;
}

export interface Question<T> {
  answeredBy(actor: Actor): Promise<T>;
}

export class Actor {
  private abilities = new Map<string, Ability>();

  constructor(public readonly name: string) {}

  whoCan(...abilities: Ability[]): this {
    for (const ability of abilities) {
      this.abilities.set(ability.constructor.name, ability);
    }
    return this;
  }

  // eslint-disable-next-line @typescript-eslint/no-unsafe-function-type
  abilityTo<T extends Ability>(type: Function & { prototype: T }): T {
    const ability = this.abilities.get(type.name);
    if (!ability) {
      throw new Error(
        `Actor "${this.name}" does not have the ability: ${type.name}. ` +
          `Add it with actor.whoCan(${type.name}.using(...))`,
      );
    }
    return ability as T;
  }

  async attemptsTo(...tasks: Task[]): Promise<void> {
    for (const task of tasks) {
      await task.performAs(this);
    }
  }

  async asks<T>(question: Question<T>): Promise<T> {
    return question.answeredBy(this);
  }
}
