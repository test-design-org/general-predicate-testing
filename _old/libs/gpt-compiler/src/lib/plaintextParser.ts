import { zip } from 'fp-ts/lib/Array';
import { Interval } from 'interval-arithmetic';
import {
  BoolDTO,
  Expression,
  IInput,
  IntervalDTO,
  MissingVariableDTO,
  createUnaryIntervalDTO,
  expressionFromString,
} from '@testing-repo/gpt-common';
import ohm from 'ohm-js';

export class BoolVariable {
  constructor(public name: string) {}
}

export class NumberVariable {
  constructor(public name: string, public precision: number) {}
}

export type Variable = BoolVariable | NumberVariable;

const gptGrammar = ohm.grammar(String.raw`
Gpt {
  Everything = VarDecls TestCases

  TestCases = NonemptyListOf<TestCase, ";">
  TestCase = "*" -- any
       | constantToken? bool -- bool
       | constantToken? unaryOp number -- unary
       | constantToken? ("(" | "[") number "," number (")" | "]") -- interval

  constantToken = "$"
  bool = "true" | "false"
  unaryOp = "<=" |  ">=" |  "!=" | "<" | ">" | "="

  VarDecls = NonemptyListOf<VarDecl, ";">
  VarDecl = varName "(" "bool" ")" -- bool
          | varName "(" "int" ")" -- int
          | varName "(" "num" ")" -- num
          | varName "(" "num" "," posNumber ")" -- numWithPrec
  varName = (letter | "_") (alnum | "_")*

  posNumber = digit+ ("." digit+)?
  number = posNumber -- pos
         | "-" posNumber -- neg
}
`);

const gptSemantics = gptGrammar.createSemantics().addOperation('eval', {
  Everything(vars, tests) {
    return [vars.eval(), tests.eval()] as const;
  },
  TestCases(tests) {
    return tests.asIteration().children.map((x) => x.eval());
  },
  TestCase_any(_star) {
    return new MissingVariableDTO();
  },
  TestCase_bool(constantToken, bool) {
    const val = bool.eval();
    const isConstant = constantToken.numChildren > 0;
    return new BoolDTO(
      val ? Expression.BoolTrue : Expression.BoolFalse,
      bool.eval(),
      isConstant,
    );
  },
  TestCase_unary(constantToken, unaryOp, number) {
    const isConstant = constantToken.numChildren > 0;
    return createUnaryIntervalDTO(
      expressionFromString(unaryOp.sourceString),
      number.eval(),
      2352345623.0, // This will be overridden
      isConstant,
    );
  },
  TestCase_interval(constantToken, lb, lo, comma, hi, rb) {
    const isConstant = constantToken.numChildren > 0;
    const isOpen = {
      lo: lb.sourceString === '(',
      hi: rb.sourceString === ')',
    };
    return new IntervalDTO(
      Expression.Interval,
      new Interval(lo.eval(), hi.eval()),
      45646545.0, // This will be overridden
      isOpen,
      isConstant,
    );
  },
  bool(bool) {
    return bool.sourceString === 'true' ? true : false;
  },
  VarDecls(vars) {
    return vars.asIteration().children.map((x) => x.eval());
  },
  VarDecl_bool(varName, lb, bool, rb) {
    return new BoolVariable(varName.eval());
  },
  VarDecl_int(varName, lb, bool, rb) {
    return new NumberVariable(varName.eval(), 1.0);
  },
  VarDecl_num(varName, lb, bool, rb) {
    return new NumberVariable(varName.eval(), 0.01);
  },
  VarDecl_numWithPrec(varName, lb, litnum, comma, precision, rb) {
    return new NumberVariable(varName.eval(), precision.eval());
  },
  varName(letter, alnum) {
    return this.sourceString;
  },
  posNumber(_digits, _decimal, _digits2) {
    return parseFloat(this.sourceString);
  },
  number_pos(number) {
    return number.eval();
  },
  number_neg(_minus, number) {
    return parseFloat(`-${number.eval()}`);
  },
} as any);

export function parseVariables(varString: string): Variable[] {
  const m = gptGrammar.match(varString, 'VarDecls');
  if (m.succeeded()) {
    const parse = gptSemantics(m)['eval']();
    console.log(parse);
    return parse;
  } else {
    throw new Error(m.message);
  }
}

export function parseTestCases(variables: Variable[], line: string): IInput[] {
  const m = gptGrammar.match(line, 'TestCases');
  if (m.succeeded()) {
    const parsedInputs = gptSemantics(m)['eval']();

    const validatedVariableInputs = zip(variables)(parsedInputs).map(
      ([input, variable]) => {
        if (
          !(input instanceof MissingVariableDTO) &&
          !(
            (variable instanceof NumberVariable &&
              input instanceof IntervalDTO) ||
            (variable instanceof BoolVariable && input instanceof BoolDTO)
          )
        ) {
          throw new Error(
            'Type error: Test case and variable type has incompatible types',
          );
        }

        if (variable instanceof NumberVariable) {
          return input.withPrecision(variable.precision);
        }

        return input;
      },
    );

    return validatedVariableInputs;
  } else {
    throw new Error(m.message);
  }
}

export function parseInput(input: string): [Variable[], IInput[][]] {
  const lines = input
    .trim()
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => !line.startsWith('//') && line !== '');

  const variables = parseVariables(lines[0]);

  const testCases = lines.slice(1);
  const inputs = testCases.map((x) => parseTestCases(variables, x));

  return [variables, inputs];
}
