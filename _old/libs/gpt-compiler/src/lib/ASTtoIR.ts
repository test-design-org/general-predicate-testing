import { Expression, MultiInterval } from '@testing-repo/gpt-common';
import {
  BinaryCondition,
  BinaryOp,
  BoolCondition,
  Condition,
  ConditionsNode,
  EqOp,
  FeatureNode,
  IfNode,
  IntervalCondition,
  VarNode,
} from './AST';
import { GPT } from './IR';

const mapBinaryOpToExpression: { [key in BinaryOp]: Expression } = {
  '=': Expression.EqualTo,
  '!=': Expression.NotEqualTo,
  '<=': Expression.LessThanOrEqualTo,
  '>=': Expression.GreaterThanOrEqualTo,
  '<': Expression.LessThan,
  '>': Expression.GreaterThan,
};

const flipBinaryOp: { [key in BinaryOp]: BinaryOp } = {
  '=': '=',
  '!=': '!=',
  '<=': '>=',
  '>=': '<=',
  '<': '>',
  '>': '<',
};

export const resolveBoolCondition = (eqOp: EqOp, boolVal: boolean): boolean => {
  /*
  x = true    ->  true
  x != true   ->  false
  x = false   ->  false
  x != false  ->  true
  */
  switch (eqOp) {
    case '=':
      return boolVal;

    case '!=':
      return !boolVal;
  }
};

export const convertBoolCondition = (cond: BoolCondition): GPT.Condition => {
  const shouldEqualTo = resolveBoolCondition(cond.eqOp, cond.boolVal);

  return { type: 'bool', varName: cond.varName, shouldEqualTo };
};

const convertBinaryCondition = (cond: BinaryCondition): GPT.Condition => {
  // createUnaryIntervalDTO expects the constant to be on the right, like: x < 0
  // If it was inputted on the left like 0 > x we should flip it to be x < 0
  const binaryOp: BinaryOp =
    cond.constantPosition === 'lhs'
      ? flipBinaryOp[cond.constantPosition]
      : cond.binaryOp;

  const expression = mapBinaryOpToExpression[binaryOp];

  return {
    type: 'interval',
    varName: cond.varName,
    expression,
    interval: MultiInterval.fromUnaryExpression(expression, cond.constant),
  };
};

const convertIntervalCondition = (cond: IntervalCondition): GPT.Condition => {
  return {
    type: 'interval',
    varName: cond.varName,
    expression: Expression.Interval,
    interval: MultiInterval.simple(
      cond.interval.interval,
      cond.interval.isOpen,
    ),
  };
};

const convertConditionNode = (conds: ConditionsNode): GPT.Predicate => {
  return conds.conditions.map((cond) => {
    switch (cond.type) {
      case 'bool':
        return convertBoolCondition(cond);

      case 'binary':
        return convertBinaryCondition(cond);

      case 'interval':
        return convertIntervalCondition(cond);
    }
  });
};

const traverseIfNode = (ifNode: IfNode): GPT.Predicate[] => {
  if (ifNode.elseIf?.length > 0) {
    throw new Error('Else if not yet supported!');
  }
  if (ifNode.elseNode !== undefined) {
    throw new Error('Else not yet supported!');
  }

  const initialConditions = convertConditionNode(ifNode.conditions);

  if (ifNode.body === undefined) {
    return [initialConditions];
  }

  const bodyConditions = ifNode.body.map(traverseIfNode).flat();

  return bodyConditions.map((x) => [...initialConditions, ...x]);
};

const convertVariable = (varNode: VarNode): GPT.Variable => {
  switch (varNode.varType.type) {
    case 'bool':
      return { varName: varNode.varName, type: { type: 'bool' } };

    case 'number':
      return {
        varName: varNode.varName,
        type: { type: 'number', precision: varNode.varType.precision },
      };
  }
};

const traverseFeatureNode = (
  featureNode: FeatureNode,
): [GPT.Variable[], GPT.Predicate[]] => {
  if (featureNode.features.length > 0) {
    throw new Error('Nested features are not yet implemented.');
  }

  const variables = featureNode.variables.map(convertVariable);
  const predicates = featureNode.ifStatements.map(traverseIfNode).flat();

  return [variables, predicates];
};

export const traverseASTtoIR = (
  ast: FeatureNode,
): [GPT.Variable[], GPT.Predicate[]] => {
  return traverseFeatureNode(ast);
};
