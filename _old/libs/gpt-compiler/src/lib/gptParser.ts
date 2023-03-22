import { IntervalDTO, IsOpen, NTuple } from '@testing-repo/gpt-common';
import { Interval } from 'interval-arithmetic';
import ohm from 'ohm-js';
import {
  ASTNode,
  BinaryCondition,
  BinaryOp,
  BoolCondition,
  Condition,
  ConditionsNode,
  ElseIfNode,
  ElseNode,
  EqOp,
  FeatureNode,
  IfNode,
  IntervalCondition,
  IntervalOp,
  IntervalWithOpenness,
  VarNode,
} from './AST';
import { traverseASTtoIR } from './ASTtoIR';
import { traverseAST } from './gptASTNodeConverter';
import { IRtoNTuple } from './IRtoNTuple';
import { Variable } from './plaintextParser';

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
  intervalOp = "in"         // | "not in"
  binaryOp = "<=" |  ">=" | "!=" | "<" | ">" | "="
  boolOp = "&&"             // | "||"

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

const isVarNode = (x: ASTNode): x is VarNode => x.type === 'var';
const isIfNode = (x: ASTNode): x is IfNode => x.type === 'if';
const isFeatureNode = (x: ASTNode): x is FeatureNode => x.type === 'feature';

const gptSemantics = gptGrammar
  .createSemantics()
  .addOperation('toAST', {
    _iter(...children): any {
      return children.map((c) => c['toAST']());
    },
    Feature(_lBrace, statements, _rBrace): FeatureNode {
      const statementASTs: ASTNode[] = statements.children.map((node) =>
        node['toAST'](),
      );
      return {
        type: 'feature',
        variables: statementASTs.filter(isVarNode),
        ifStatements: statementASTs.filter(isIfNode),
        features: statementASTs.filter(isFeatureNode),
      };
    },
    VarDecl_bool(_var, varName, _colon, _bool): VarNode {
      return {
        type: 'var',
        varName: varName.sourceString,
        varType: { type: 'bool' },
      };
    },
    VarDecl_int(_var, varName, _colon, _int): VarNode {
      return {
        type: 'var',
        varName: varName.sourceString,
        varType: { type: 'number', precision: 1 },
      };
    },
    VarDecl_numWithPrec(
      _var,
      varName,
      _colon,
      _num,
      _lBrace,
      precision,
      _rBrace,
    ): VarNode {
      return {
        type: 'var',
        varName: varName.sourceString,
        varType: {
          type: 'number',
          precision: parseFloat(precision.sourceString),
        },
      };
    },
    VarDecl_num(_var, varName, _colon, _num): VarNode {
      return {
        type: 'var',
        varName: varName.sourceString,
        varType: {
          type: 'number',
          precision: 0.01,
        },
      };
    },
    IfStmt(
      _if,
      _lBrace1,
      conditions,
      _rBrace1,
      _lBrace2,
      body,
      _rBrace2,
      elseIfs,
      elseNode,
    ): IfNode {
      return {
        type: 'if',
        conditions: conditions['toAST'](),
        body: body.children.map((node) => node['toAST']())[0],
        elseIf: elseIfs.children.map((node) => node['toAST']()),
        elseNode: elseNode?.children[0]?.['toAST'](),
      };
    },
    ElseIf(
      _else,
      _if,
      _lBrace1,
      conditions,
      _rBrace1,
      _lBrace2,
      body,
      _rBrace2,
    ): ElseIfNode {
      return {
        type: 'elseIf',
        conditions: conditions['toAST'](),
        body: body.children.map((node) => node['toAST']())[0],
      };
    },
    Else(_else, _lBrace, body, _rBrace): ElseNode {
      return {
        type: 'else',
        body: body.children.map((node) => node['toAST']()),
      };
    },
    Conditions(conditions): ConditionsNode {
      return {
        type: 'conditions',
        conditions: conditions['asIteration']().children.map((node) =>
          node['toCondition'](),
        ),
      };
    },
  } as ohm.ActionDict<ASTNode>)
  .addOperation('toCondition', {
    _iter(...children): any {
      return children.map((c) => c['toCondition']());
    },
    Cond_boolLhs(boolVal, eqOp, varName): BoolCondition {
      return {
        type: 'bool',
        varName: varName.sourceString,
        eqOp: eqOp.sourceString as EqOp,
        boolVal: boolVal.sourceString === 'true',
      };
    },
    Cond_boolRhs(varName, eqOp, boolVal): BoolCondition {
      return {
        type: 'bool',
        varName: varName.sourceString,
        eqOp: eqOp.sourceString as EqOp,
        boolVal: boolVal.sourceString === 'true',
      };
    },
    Cond_binaryLhs(constantNumber, binaryOp, varName): BinaryCondition {
      return {
        type: 'binary',
        varName: varName.sourceString,
        constantPosition: 'lhs',
        constant: parseFloat(constantNumber.sourceString),
        binaryOp: binaryOp.sourceString as BinaryOp,
      };
    },
    Cond_binaryRhs(varName, binaryOp, constantNumber): BinaryCondition {
      return {
        type: 'binary',
        varName: varName.sourceString,
        constantPosition: 'rhs',
        constant: parseFloat(constantNumber.sourceString),
        binaryOp: binaryOp.sourceString as BinaryOp,
      };
    },
    Cond_interval(varName, intervalOp, interval): IntervalCondition {
      return {
        type: 'interval',
        varName: varName.sourceString,
        intervalOp: intervalOp.sourceString as IntervalOp,
        interval: interval['toInterval'](),
      };
    },
  } as ohm.ActionDict<Condition>)
  .addOperation('toInterval', {
    Interval(lBrace, lo, _comma, hi, rBrace): IntervalWithOpenness {
      return {
        interval: new Interval(
          parseFloat(lo.sourceString),
          parseFloat(hi.sourceString),
        ),
        isOpen: {
          lo: lBrace.sourceString === '(',
          hi: rBrace.sourceString === ')',
        },
      };
    },
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

export const parseGptToNTuples = (text: string): [Variable[], NTuple[]] => {
  return traverseAST(parseGpt(text));
};

export const parseGPTtoNTuplesWithIR = (
  text: string,
): [Variable[], NTuple[]] => {
  const [variables, predicates] = traverseASTtoIR(parseGpt(text));
  return IRtoNTuple(variables, predicates);
};