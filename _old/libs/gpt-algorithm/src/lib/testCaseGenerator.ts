import { array } from 'fp-ts';
import { Interval } from 'interval-arithmetic';
import {
  BoolDTO,
  Expression,
  IInput,
  IntervalDTO,
  MissingVariableDTO,
  NTuple,
} from '@testing-repo/gpt-common';

export function generateTestCases(inputs: IInput[]): NTuple[] {
  const nTuples = [
    calculateInOnPatterns1(inputs),
    calculateInOnPatterns2(inputs),
    ...OffOut(inputs),
  ].map((x) => new NTuple(x));

  return array.uniq(NTuple.Eq)(nTuples);
}

const calculateInOnPatterns1 = (inputs: IInput[]): IInput[] =>
  // eslint-disable-next-line array-callback-return
  inputs.map((input) => {
    if (input.isConstant) return input;

    switch (input.expression) {
      case Expression.LessThan:
      case Expression.LessThanOrEqualTo:
        return InIn(input, 1);

      case Expression.GreaterThan:
      case Expression.GreaterThanOrEqualTo:
        return InIn(input, 2);

      case Expression.NotEqualTo:
        return In(input, 2);

      case Expression.EqualTo:
        return On(input);

      case Expression.BoolTrue:
      case Expression.BoolFalse:
      case Expression.MissingVariable:
        return input;

      case Expression.Interval:
        return On(input, 2);
    }
  });

const calculateInOnPatterns2 = (inputs: IInput[]): IInput[] =>
  // eslint-disable-next-line array-callback-return
  inputs.map((input) => {
    if (input.isConstant) return input;

    switch (input.expression) {
      case Expression.LessThan:
      case Expression.LessThanOrEqualTo:
        return On(input, 1);

      case Expression.EqualTo:
        return On(input);

      case Expression.GreaterThan:
      case Expression.GreaterThanOrEqualTo:
        return On(input, 2);

      case Expression.NotEqualTo:
        return In(input, 1);

      case Expression.BoolTrue:
      case Expression.BoolFalse:
      case Expression.MissingVariable:
        return input;

      case Expression.Interval:
        return On(input, 1);
    }
  });

const baseline = (inputs: IInput[]): IInput[] =>
  // eslint-disable-next-line array-callback-return
  inputs.map((input) => {
    if (input.isConstant) return input;

    switch (input.expression) {
      case Expression.LessThan:
      case Expression.LessThanOrEqualTo:
        return In(input, 1);

      case Expression.GreaterThan:
      case Expression.GreaterThanOrEqualTo:
      case Expression.NotEqualTo:
        return In(input, 2);

      case Expression.EqualTo:
        return On(input);

      case Expression.BoolTrue:
      case Expression.BoolFalse:
      case Expression.MissingVariable:
        return input;

      case Expression.Interval:
        return In(input, 3);
    }
  });

function OffOut(inputs: IInput[]): IInput[][] {
  const output: IInput[][] = [];

  for (let i = 0; i < inputs.length; ++i) {
    if (inputs[i] instanceof MissingVariableDTO) continue;
    if (inputs[i].isConstant) continue;

    const based1 = baseline(inputs);
    const based2 = baseline(inputs);
    const based3 = baseline(inputs);
    const based4 = baseline(inputs);

    switch (inputs[i].expression) {
      case Expression.LessThan:
      case Expression.LessThanOrEqualTo:
        based1[i] = Out(inputs[i], 1);
        based2[i] = Off(inputs[i], 1);

        output.push(based1);
        output.push(based2);
        break;

      case Expression.GreaterThan:
      case Expression.GreaterThanOrEqualTo:
        based1[i] = Out(inputs[i], 2);
        based2[i] = Off(inputs[i], 2);

        output.push(based1);
        output.push(based2);
        break;

      case Expression.EqualTo:
        based1[i] = Out(inputs[i], 3);
        based2[i] = Out(inputs[i], 4);

        output.push(based1);
        output.push(based2);
        break;

      case Expression.NotEqualTo:
        based1[i] = Off(inputs[i]);

        output.push(based1);
        break;

      case Expression.BoolTrue:
        based1[i] = new BoolDTO(inputs[i].expression, false);
        output.push(based1);
        break;

      case Expression.BoolFalse:
        based1[i] = new BoolDTO(inputs[i].expression, true);
        output.push(based1);
        break;

      case Expression.Interval:
        based1[i] = Out(inputs[i], 1);
        based2[i] = Out(inputs[i], 2);
        based3[i] = Off(inputs[i], 1);
        based4[i] = Off(inputs[i], 2);

        output.push(based1);
        output.push(based4);
        output.push(based3);
        output.push(based2);
        break;
    }
  }

  return output;
}

function On(input: IInput, version: 0 | 1 | 2 = 0): IInput {
  if (!(input instanceof IntervalDTO))
    throw new Error("On's argument must be an IntervalDTO");

  switch (version) {
    // <, <=, Interval Right
    case 1:
      return new IntervalDTO(
        input.expression,
        new Interval(
          input.interval.hi - (input.isOpen.hi ? 1 : 0) * input.precision,
          input.interval.hi - (input.isOpen.hi ? 1 : 0) * input.precision,
        ),
        input.precision,
      );
    // >, >=, Interval left
    case 2:
      return new IntervalDTO(
        input.expression,
        new Interval(
          input.interval.lo + (input.isOpen.lo ? 1 : 0) * input.precision,
          input.interval.lo + (input.isOpen.lo ? 1 : 0) * input.precision,
        ),
        input.precision,
      );
    // ==
    case 0:
      return new IntervalDTO(input.expression, input.interval, input.precision);
  }
}

function In(input: IInput, version: 1 | 2 | 3): IInput {
  if (!(input instanceof IntervalDTO))
    throw new Error("In's argument must be an IntervalDTO");

  switch (version) {
    // <, <=
    case 1:
      return new IntervalDTO(
        input.expression,
        new Interval(
          -Infinity,
          input.interval.hi - (input.isOpen.hi ? 1 : 0) * input.precision,
        ),
        input.precision,
      );
    // >, =>
    case 2:
      return new IntervalDTO(
        input.expression,
        new Interval(
          input.interval.lo + (input.isOpen.lo ? 1 : 0) * input.precision,
          Infinity,
        ),
        input.precision,
      );
    // Interval
    case 3:
      return new IntervalDTO(
        input.expression,
        new Interval(
          input.interval.lo + (input.isOpen.lo ? 1 : 0) * input.precision,
          input.interval.hi - (input.isOpen.hi ? 1 : 0) * input.precision,
        ),
        input.precision,
      );
  }
}

function InIn(input: IInput, version: 1 | 2): IInput {
  if (!(input instanceof IntervalDTO))
    throw new Error("InIn's argument must be an IntervalDTO");

  switch (version) {
    // <, <=
    case 1:
      return new IntervalDTO(
        input.expression,
        new Interval(
          -Infinity,
          input.interval.hi - (input.isOpen.hi ? 2 : 1) * input.precision,
        ),
        input.precision,
      );
    // >, =>
    case 2:
      return new IntervalDTO(
        input.expression,
        new Interval(
          input.interval.lo + (input.isOpen.lo ? 2 : 1) * input.precision,
          Infinity,
        ),
        input.precision,
      );
  }
}

function Off(input: IInput, version = 0): IInput {
  if (!(input instanceof IntervalDTO))
    throw new Error("Off's argument must be an IntervalDTO");

  switch (version) {
    // <, <=, Interval Right, == right
    case 1:
      return new IntervalDTO(
        input.expression,
        new Interval(
          input.interval.hi + (input.isOpen.hi ? 0 : 1) * input.precision,
        ),
        input.precision,
      );
    // >, >=, // Interval left, == left
    case 2:
      return new IntervalDTO(
        input.expression,
        new Interval(
          input.interval.lo - (input.isOpen.lo ? 0 : 1) * input.precision,
        ),
        input.precision,
      );

    default:
      return new IntervalDTO(input.expression, input.interval, input.precision);
  }
}

function Out(input: IInput, version: 1 | 2 | 3 | 4): IInput {
  if (!(input instanceof IntervalDTO))
    throw new Error("Off's argument must be an IntervalDTO");

  switch (version) {
    // <, <=, Interval Right
    case 1:
      return new IntervalDTO(
        input.expression,
        new Interval(
          input.interval.hi + (input.isOpen.hi ? 1 : 2) * input.precision,
          Infinity,
        ),
        input.precision,
      );

    // >, =>, Interval Left
    case 2:
      return new IntervalDTO(
        input.expression,
        new Interval(
          -Infinity,
          input.interval.lo - (input.isOpen.lo ? 1 : 2) * input.precision,
        ),
        input.precision,
      );
    // =, Right
    case 3:
      return new IntervalDTO(
        input.expression,
        new Interval(input.interval.hi + input.precision, Infinity),
        input.precision,
      );
    // =, Left
    case 4:
      return new IntervalDTO(
        input.expression,
        new Interval(-Infinity, input.interval.lo - input.precision),
        input.precision,
      );
  }
}
