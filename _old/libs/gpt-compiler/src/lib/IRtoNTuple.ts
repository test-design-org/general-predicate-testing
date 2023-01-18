// We make the assumption, that
//  - everything is type checked,
//  - each predicate has a reference to each variable at most once
//    - NTuples have each variable once, that's why

import {
  BoolDTO,
  Expression,
  IInput,
  IntervalDTO,
  MissingVariableDTO,
  NTuple,
} from '@testing-repo/gpt-common';
import { GPT } from './IR';
import { BoolVariable, NumberVariable, Variable } from './plaintextParser';

export const convertBoolDTO = (condition: GPT.BoolCondition): BoolDTO => {
  const expression =
    condition.shouldEqualTo === true
      ? Expression.BoolTrue
      : Expression.BoolFalse;
  return new BoolDTO(expression, condition.shouldEqualTo, false); // TODO: isConstant should be provided
};

export const convertIntervalDTO = (
  variable: GPT.Variable,
  condition: GPT.IntervalCondition,
): IntervalDTO => {
  const precision = (variable.type as GPT.NumberType).precision;
  const [isOpen, interval] = condition.interval.DONTUSE_getInterval();
  return new IntervalDTO(
    condition.expression,
    interval,
    precision,
    isOpen,
    false,
  ); // TODO: isConstant should be provided
};

export const convertCondition = (
  variable: GPT.Variable,
  condition: GPT.Condition,
): IInput => {
  switch (condition.type) {
    case 'bool':
      return convertBoolDTO(condition);

    case 'interval':
      return convertIntervalDTO(variable, condition);
  }
};

export const sortObjectsIntoTuple = <TTupleElem, TObject, TCompare>(
  tupleFormat: TTupleElem[],
  objects: TObject[],
  lensTupleElem: (_x: TTupleElem) => TCompare,
  lensObject: (_x: TObject) => TCompare,
): [TTupleElem, TObject | undefined][] =>
  tupleFormat.map((tupleElem) => [
    tupleElem,
    objects.find((x) => lensObject(x) === lensTupleElem(tupleElem)),
  ]);

export const convertPredicateToNTuple = (
  variables: GPT.Variable[],
  predicate: GPT.Predicate,
): NTuple => {
  const iinputs: IInput[] = sortObjectsIntoTuple(
    variables,
    predicate,
    (x) => x.varName,
    (x) => x.varName,
  ).map(([variable, condition]) =>
    condition === undefined
      ? new MissingVariableDTO()
      : convertCondition(variable, condition),
  );

  return new NTuple(iinputs);
};

export const convertVariable = (variable: GPT.Variable): Variable => {
  switch (variable.type.type) {
    case 'bool':
      return new BoolVariable(variable.varName);

    case 'number':
      return new NumberVariable(variable.varName, variable.type.precision);
  }
};

export const IRtoNTuple = (
  variables: GPT.Variable[],
  predicates: GPT.Predicate[],
): [Variable[], NTuple[]] => {
  return [
    variables.map(convertVariable),
    predicates.map((predicate) =>
      convertPredicateToNTuple(variables, predicate),
    ),
  ];
};
