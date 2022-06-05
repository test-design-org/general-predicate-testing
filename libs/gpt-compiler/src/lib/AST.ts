import { IsOpen } from '@testing-repo/gpt-common';
import { Interval } from 'interval-arithmetic';

export class VarType {}
export class BoolType extends VarType {}
export class NumType implements VarType {
  constructor(public precision: number) {}

  static integer() {
    return new NumType(1);
  }
}

export class FeatureNode {
  constructor(
    public variables: VarNode[],
    public ifStatements: IfNode[],
    public features: FeatureNode[],
  ) {}
}

export class VarNode {
  constructor(public varName: string, public varType: VarType) {}
}

export class IfNode {
  constructor(
    public conditions: ConditionsNode,
    public body: IfNode[] | undefined,
    public elseIf: ElseIfNode[],
    public elseNode: ElseNode | undefined,
  ) {}
}

export class ElseIfNode {
  constructor(
    public conditions: ConditionsNode,
    public body: IfNode[] | undefined,
  ) {}
}

export class ElseNode {
  constructor(public body: IfNode[]) {}
}

export class ConditionsNode {
  constructor(public conditions: Condition[]) {}
}

export type EqOp = '=' | '!=';
export type BinaryOp = '<=' | '>=' | '!=' | '<' | '>' | '=';
export type IntervalOp = 'in' /*| 'not in' */;

export class Condition {}
export class BoolCondition extends Condition {
  constructor(
    public varName: string,
    public eqOp: EqOp,
    public boolVal: boolean,
  ) {
    super();
  }
}

export class BinaryCondition extends Condition {
  constructor(
    public constantPosition: 'lhs' | 'rhs',
    public constant: number,
    public binaryOp: BinaryOp,
    public varName: string,
  ) {
    super();
  }
}

export class IntervalCondition extends Condition {
  constructor(
    public varName: string,
    public intervalOp: IntervalOp,
    public interval: IntervalWithOpenness,
  ) {
    super();
  }
}

export class IntervalWithOpenness {
  constructor(public interval: Interval, public isOpen: IsOpen) {}
}

export type ASTNode =
  | FeatureNode
  | VarNode
  | IfNode
  | ElseIfNode
  | ElseNode
  | ConditionsNode;
