import { IntervalDTO, IsOpen } from '@testing-repo/gpt-common';
import { Interval } from 'interval-arithmetic';
import ohm from 'ohm-js';
import ohmExtras from 'ohm-js/extras';
import { SuperProperty } from 'typescript';

class VarType {}
class BoolType extends VarType {}
class NumType implements VarType {
  constructor(public precision: number) {}

  static integer() {
    return new NumType(1);
  }
}

class FeatureNode {
  constructor(
    public variables: VarNode[],
    public ifStatements: IfNode[],
    public features: FeatureNode[],
  ) {}
}

class VarNode {
  constructor(public varName: string, public varType: VarType) {}
}

class IfNode {
  constructor(
    public conditions: ConditionsNode,
    public body: IfNode[],
    public elseIf: ElseIfNode[],
    public elseNode?: ElseNode,
  ) {}
}

class ElseIfNode {
  constructor(public conditions: ConditionsNode, public body: IfNode[]) {}
}

class ElseNode {
  constructor(public body: IfNode[]) {}
}

class ConditionsNode {
  constructor(public conditions: Condition[]) {}
}

type EqOp = '=' | '!=';
type BinaryOp = '<=' | '>=' | '!=' | '<' | '>' | '=';
type IntervalOp = 'in' | 'not in';

class Condition {}
class BoolCondition extends Condition {
  constructor(
    public varName: string,
    public eqOp: EqOp,
    public boolVal: boolean,
  ) {
    super();
  }
}

class BinaryCondition extends Condition {
  constructor(
    public constantPosition: 'lhs' | 'rhs',
    public constant: number,
    public binaryOp: BinaryOp,
    public varName: string,
  ) {
    super();
  }
}

class IntervalCondition extends Condition {
  constructor(
    public varName: string,
    public intervalOp: IntervalOp,
    public interval: IntervalWithOpenness,
  ) {
    super();
  }
}

class IntervalWithOpenness {
  constructor(public interval: Interval, public isOpen: IsOpen) {}
}

type ASTNode =
  | FeatureNode
  | VarNode
  | IfNode
  | ElseIfNode
  | ElseNode
  | ConditionsNode;

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

const isVarNode = (x: ASTNode): x is VarNode => x instanceof VarNode;
const isIfNode = (x: ASTNode): x is IfNode => x instanceof IfNode;
const isFeatureNode = (x: ASTNode): x is FeatureNode =>
  x instanceof FeatureNode;

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
      return new FeatureNode(
        statementASTs.filter(isVarNode),
        statementASTs.filter(isIfNode),
        statementASTs.filter(isFeatureNode),
      );
    },
    VarDecl_bool(_var, varName, _colon, _bool): VarNode {
      return new VarNode(varName.sourceString, new BoolType());
    },
    VarDecl_int(_var, varName, _colon, _int): VarNode {
      return new VarNode(varName.sourceString, NumType.integer());
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
      return new VarNode(
        varName.sourceString,
        new NumType(parseFloat(precision.sourceString)),
      );
    },
    VarDecl_num(_var, varName, _colon, _num): VarNode {
      return new VarNode(varName.sourceString, new NumType(0.01));
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
      return new IfNode(
        conditions['toAST'](),
        body.children.map((node) => node['toAST']()),
        elseIfs.children.map((node) => node['toAST']()),
        elseNode.children[0]?.['toAST'](),
      );
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
      return new ElseIfNode(
        conditions['toAST'](),
        body.children.map((node) => node['toAST']()),
      );
    },
    Else(_else, _lBrace, body, _rBrace): ElseNode {
      return new ElseNode(body.children.map((node) => node['toAST']()));
    },
    Conditions(conditions): ConditionsNode {
      return new ConditionsNode(
        conditions['asIteration']().children.map((node) =>
          node['toCondition'](),
        ),
      );
    },
  } as ohm.ActionDict<ASTNode>)
  .addOperation('toCondition', {
    _iter(...children) {
      return children.map((c) => c['toCondition']());
    },
    Cond_boolLhs(boolVal, eqOp, varName): BoolCondition {
      return new BoolCondition(
        varName.sourceString,
        eqOp.sourceString as EqOp,
        boolVal.sourceString === 'true',
      );
    },
    Cond_boolRhs(varName, eqOp, boolVal): BoolCondition {
      return new BoolCondition(
        varName.sourceString,
        eqOp.sourceString as EqOp,
        boolVal.sourceString === 'true',
      );
    },
    Cond_binaryLhs(constantNumber, binaryOp, varName): BinaryCondition {
      return new BinaryCondition(
        'lhs',
        parseFloat(constantNumber.sourceString),
        binaryOp.sourceString as BinaryOp,
        varName.sourceString,
      );
    },
    Cond_binaryRhs(varName, binaryOp, constantNumber): BinaryCondition {
      return new BinaryCondition(
        'rhs',
        parseFloat(constantNumber.sourceString),
        binaryOp.sourceString as BinaryOp,
        varName.sourceString,
      );
    },
    Cond_interval(varName, intervalOp, interval): IntervalCondition {
      return new IntervalCondition(
        varName.sourceString,
        intervalOp.sourceString as IntervalOp,
        interval['toInterval'](),
      );
    },
  } as ohm.ActionDict<Condition>)
  .addOperation('toInterval', {
    Interval(lBrace, lo, _comma, hi, rBrace): IntervalWithOpenness {
      return new IntervalWithOpenness(
        new Interval(parseFloat(lo.sourceString), parseFloat(hi.sourceString)),
        {
          lo: lBrace.sourceString === '(',
          hi: rBrace.sourceString === ')',
        },
      );
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
