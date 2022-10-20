import { IsOpen } from '@testing-repo/gpt-common';
import { Interval } from 'interval-arithmetic';

export type BoolType = { type: 'bool' };
export type NumberType = { type: 'number'; precision: number };
export type VarType = BoolType | NumberType;

export type EqOp = '=' | '!=';
export type BinaryOp = '<=' | '>=' | '!=' | '<' | '>' | '=';
export type IntervalOp = 'in' /*| 'not in' */;

export type BoolCondition = {
  type: 'bool';
  varName: string;
  eqOp: EqOp;
  boolVal: boolean;
};

export type BinaryCondition = {
  type: 'binary';
  varName: string;
  constantPosition: 'lhs' | 'rhs';
  constant: number;
  binaryOp: BinaryOp;
};

export type IntervalCondition = {
  type: 'interval';
  varName: string;
  intervalOp: IntervalOp;
  interval: IntervalWithOpenness;
};

export type Condition = BoolCondition | BinaryCondition | IntervalCondition;

export type IntervalWithOpenness = {
  interval: Interval;
  isOpen: IsOpen;
};

export type FeatureNode = {
  type: 'feature';
  variables: VarNode[];
  ifStatements: IfNode[];
  features: FeatureNode[];
};

export type VarNode = {
  type: 'var';
  varName: string;
  varType: VarType;
};

export type IfNode = {
  type: 'if';
  conditions: ConditionsNode;
  body?: IfNode[];
  elseIf: ElseIfNode[];
  elseNode?: ElseNode;
};

export type ElseIfNode = {
  type: 'elseIf';
  conditions: ConditionsNode;
  body?: IfNode[];
};

export type ElseNode = {
  type: 'else';
  body: IfNode[];
};

export type ConditionsNode = {
  type: 'conditions';
  conditions: Condition[];
};

export type ASTNode =
  | FeatureNode
  | VarNode
  | IfNode
  | ElseIfNode
  | ElseNode
  | ConditionsNode;
