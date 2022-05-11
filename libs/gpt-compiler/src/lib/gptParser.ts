import { IntervalDTO, IsOpen } from '@testing-repo/gpt-common';
import { Interval } from 'interval-arithmetic';
import ohm from 'ohm-js';
import ohmExtras from 'ohm-js/extras';

interface VarType {}
interface BoolType extends VarType {}
class NumType implements VarType {
  constructor(public precision: number) {}

  static integer() {
    return new NumType(1);
  }
}

interface FeatureNode {
  variables: VarNode[];
  ifStatements: IfNode[];
  features: FeatureNode[];
}

interface VarNode {
  varName: string;
  varType: VarType;
}

interface IfNode {
  conditions: ConditionsNode;
  body?: IfNode[];
  elseIf?: ElseIfNode[];
  else?: ElseNode;
}

interface ElseIfNode {
  conditions: ConditionsNode;
  body?: IfNode[];
}

interface ElseNode {
  body?: IfNode[];
}

interface ConditionsNode {
  conditions: Condition[];
}

type EqOp = '=' | '!=';
type BinaryOp = '<=' | '>=' | '!=' | '<' | '>' | '=';
type IntervalOp = 'in' | 'not in';

interface Condition {}
interface BoolCondition extends Condition {
  varName: string;
  eqOp: EqOp;
  boolVal: boolean;
}

interface BinaryCondition extends Condition {
  constantPosition: 'lhs' | 'rhs';
  constant: number;
  binaryOp: BinaryOp;
  varName: string;
}

interface IntervalCondition extends Condition {
  varName: string;
  intervalOp: IntervalOp;
  interval: IntervalWithOpenness;
}

interface IntervalWithOpenness {
  interval: Interval;
  isOpen: IsOpen;
}

type ASTNode = FeatureNode | VarNode;

const gptGrammar = ohm.grammar(String.raw`
Gpt {
  Feature = "[" (VarDecl | IfStmt | Feature)+ "]"

  VarDecl = "var" varName ":" "bool" -- bool
          | "var" varName ":" "int" -- int
          | "var" varName ":" "num" "(" posNumber ")" -- numWithPrec
          | "var" varName ":" "num" -- num
  varName = (letter | "_") (alnum | "_")*

  IfStmt = "if" "(" Conditions ")" ("{" IfStmt* "}")? ElseIf* Else?
  ElseIf = "else" "if" "(" Conditions ")" ("{" IfStmt* "}")?
  Else = "else" "{" IfStmt* "}"

  Conditions = NonemptyListOf<Cond, boolOp>
  Cond = bool eqOp varName -- boolLhs
       | varName eqOp bool -- boolRhs
       | number binaryOp varName -- binaryLhs
       | varName binaryOp number -- binaryRhs
       | varName intervalOp Interval -- interval

  Interval = ("(" | "[") number "," number (")" | "]")

  bool = "true" | "false"
  eqOp = "=" | "!="
  intervalOp = "in" | "not in"
  binaryOp = "<=" |  ">=" | "!=" | "<" | ">" | "="
  boolOp = "&&" | "||"

  posNumber = digit+ ("." digit+)?
  number = posNumber -- pos
         | "-" posNumber -- neg
         | "Inf" -- inf
         | "-" spaces "Inf" -- negInf

  comment = "/*" (~"*/" any)* "*/"
          | "//" (~"\n" any)* "\n"?
  space += comment
}
`);

const gptSemantics = gptGrammar.createSemantics().addOperation('toAST', {
  Feature(_lBrace, statements, _rBrace): FeatureNode {
    const statementASTs: ASTNode = statements['toAST']();
    return ({
      variables: 
    });
  },
  VarDecl_bool(_var, varName, _colon, _bool): 
});

export const parseGpt = (text: string) => {
  const match = gptGrammar.match(text);
  if (match.succeeded()) {
    const AST = gptSemantics(match)['toAST']();
    return AST;
  } else {
    throw new Error(match.message);
  }
};
