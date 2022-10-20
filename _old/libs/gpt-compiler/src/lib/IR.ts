// Intermediary Representation of the GPT Language

import { Expression } from '@testing-repo/gpt-common';
import { MultiInterval } from '@testing-repo/gpt-common';

// eslint-disable-next-line @typescript-eslint/no-namespace
export namespace GPT {
  export type BoolType = { type: 'bool' };
  export type NumberType = { type: 'number'; precision: number };
  export type Type = BoolType | NumberType;

  export type Variable = {
    varName: string;
    type: Type;
  };

  export type State = {
    variables: Variable[];
  };

  export type BoolCondition = {
    type: 'bool';
    varName: string;
    shouldEqualTo: boolean;
  };

  export type IntervalCondition = {
    type: 'interval';
    varName: string;
    expression: Expression;
    interval: MultiInterval;
  };

  export type Condition = BoolCondition | IntervalCondition;

  export type Predicate = Condition[];
}
