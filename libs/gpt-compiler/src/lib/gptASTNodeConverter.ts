/*

TODO: type checker
TODO: isConstant
TODO: what if a var apears multiple times in one condition?
  -> semantic checker
  -> even when rolling up multiple ifs
        if(x > 10) {
          if(x < 0);
        }
        --> should be an error

TODO: Intervals should handle more intervals
      if(x in [0,10]);
      else (...);
      
      -> Here when we traverse the 'if' node, the 'else' should have an !(x in [0, 10]) prepended to it
         but it is not currently supported

TODO: Nested features

  x    y    z    a
(..., ..., ..., ...)

*/

import {
  BoolDTO,
  createUnaryIntervalDTO,
  Expression,
  IInput,
  IntervalDTO,
  MissingVariableDTO,
  NTuple,
} from '@testing-repo/gpt-common';
import {
  ASTNode,
  BinaryCondition,
  BinaryOp,
  BoolCondition,
  BoolType,
  ConditionsNode,
  EqOp,
  FeatureNode,
  IfNode,
  IntervalCondition,
  NumType,
  VarNode,
  VarType,
} from './AST';
import { BoolVariable, NumberVariable, Variable } from './plaintextParser';

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

const mapEqOpToExpression = (
  eqOp: EqOp,
  boolVal: boolean,
): [Expression, boolean] => {
  /*
  x = true                -> BoolTrue true
  x != true   -> = false  -> BoolFalse false
  x = false               -> BoolFalse false
  x != false  -> = true   -> BoolTrue true
  */
  switch (eqOp) {
    case '=':
      if (boolVal === true) {
        return [Expression.BoolTrue, true];
      } else {
        return [Expression.BoolFalse, false];
      }

    case '!=':
      if (boolVal === true) {
        return [Expression.BoolFalse, false];
      } else {
        return [Expression.BoolTrue, true];
      }
  }
};

const convertBoolCondition = (
  cond: BoolCondition,
): { varName: string; iinput: BoolDTO } => {
  const [expression, boolVal] = mapEqOpToExpression(cond.eqOp, cond.boolVal);

  return {
    varName: cond.varName,
    iinput: new BoolDTO(expression, boolVal, false),
  };
};

const convertBinaryCondition = (
  cond: BinaryCondition,
): { varName: string; iinput: IntervalDTO } => {
  // createUnaryIntervalDTO expects the constant to be on the right, like: x < 0
  // If it was inputted on the left like 0 > x we should flip it to be x < 0
  const binaryOp: BinaryOp =
    cond.constantPosition === 'lhs'
      ? flipBinaryOp[cond.constantPosition]
      : cond.binaryOp;

  return {
    varName: cond.varName,
    iinput: createUnaryIntervalDTO(
      mapBinaryOpToExpression[binaryOp],
      cond.constant,
      0.0000069, // TODO: get precision from variable
      false,
    ),
  };
};

const convertIntervalCondition = (
  cond: IntervalCondition,
): { varName: string; iinput: IntervalDTO } => {
  return {
    varName: cond.varName,
    iinput: new IntervalDTO(
      Expression.Interval,
      cond.interval.interval,
      0.0000000069, // TODO: get precision from variable
      cond.interval.isOpen,
      false,
    ),
  };
};

const convertConditionNode = (
  conds: ConditionsNode,
): { varName: string; iinput: IInput }[] => {
  return conds.conditions.map((cond) => {
    if (cond instanceof BoolCondition) {
      return convertBoolCondition(cond);
    }

    if (cond instanceof BinaryCondition) {
      return convertBinaryCondition(cond);
    }

    if (cond instanceof IntervalCondition) {
      return convertIntervalCondition(cond);
    }

    throw new Error('Condition is not an instance of a valid type');
  });
};

const traverseIfNode = (
  ifNode: IfNode,
): { varName: string; iinput: IInput }[][] => {
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

export const traverseFeatureNodeForVars = (
  featureNode: FeatureNode,
): VarNode[] => {
  return [
    ...featureNode.variables,
    ...featureNode.features.map(traverseFeatureNodeForVars).flat(),
  ];
};

const traverseFeatureNode = (
  featureNode: FeatureNode,
): [VarNode[], NTuple[]] => {
  if (featureNode.features.length > 0) {
    throw new Error('Nested features are not yet implemented.');
  }

  // The variables will be inserted into the nTuple in this order
  const varNodes = traverseFeatureNodeForVars(featureNode);

  const nTuples = featureNode.ifStatements
    .map((ifStmt) => {
      const multipleIinputs = traverseIfNode(ifStmt);

      return multipleIinputs.map((iinputs) => {
        const orderedIInputs = varNodes.map((variable) => {
          const iinputsForVar = iinputs.filter(
            (iinput) => iinput.varName === variable.varName,
          );

          if (iinputsForVar.length > 1) {
            throw new Error(
              `Multiple conditions given for var ${variable.varName}, it is not yet implemented to give it multiple conditions.`,
            );
          }
          if (iinputsForVar.length === 0) return new MissingVariableDTO();
          // iinputsForVar.length === 1
          else {
            return iinputsForVar[0].iinput;
          }
        });

        return new NTuple(orderedIInputs);
      });
    })
    .flat();

  return [varNodes, nTuples];
};

const convertVariable = (varNode: VarNode): Variable => {
  if (varNode.varType instanceof BoolType) {
    return new BoolVariable(varNode.varName);
  }

  if (varNode.varType instanceof NumType) {
    return new NumberVariable(varNode.varName, varNode.varType.precision);
  }

  throw new Error('Unknown VarType passed to convertVariable');
};

export const traverseAST = (ast: FeatureNode): [Variable[], NTuple[]] => {
  const [varNodes, nTuples] = traverseFeatureNode(ast);
  return [varNodes.map(convertVariable), nTuples];
};
