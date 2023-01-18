import { Interval } from 'interval-arithmetic';
import { Expression, IntervalDTO } from './dtos';

export function expressionFromString(s: string): Expression {
  switch (s) {
    case '<':
      return Expression.LessThan;
    case '<=':
      return Expression.LessThanOrEqualTo;
    case '>':
      return Expression.GreaterThan;
    case '>=':
      return Expression.GreaterThanOrEqualTo;
    case '=':
      return Expression.EqualTo;
    case '!=':
      return Expression.NotEqualTo;
    default:
      throw new Error(`Cannot create an Expression from '${s}''`);
  }
}

export function createUnaryIntervalDTO(
  expression: Expression,
  num: number,
  precision: number,
  isConstant: boolean,
): IntervalDTO {
  switch (expression) {
    case Expression.LessThan:
      return new IntervalDTO(
        expression,
        new Interval(-Infinity, num),
        precision,
        { lo: true, hi: true },
        isConstant,
      );

    case Expression.LessThanOrEqualTo:
      return new IntervalDTO(
        expression,
        new Interval(-Infinity, num),
        precision,
        { lo: true, hi: false },
        isConstant,
      );

    case Expression.GreaterThan:
      return new IntervalDTO(
        expression,
        new Interval(num, Infinity),
        precision,
        { lo: true, hi: true },
        isConstant,
      );

    case Expression.GreaterThanOrEqualTo:
      return new IntervalDTO(
        expression,
        new Interval(num, Infinity),
        precision,
        { lo: false, hi: true },
        isConstant,
      );

    case Expression.EqualTo:
      return new IntervalDTO(
        expression,
        new Interval(num, num),
        precision,
        {
          lo: false,
          hi: false,
        },
        isConstant,
      );

    case Expression.NotEqualTo:
      return new IntervalDTO(
        expression,
        new Interval(num, num),
        precision,
        {
          lo: false,
          hi: false,
        },
        isConstant,
      );

    default:
      throw new Error(
        `Cannot create a unary IntervalDTO from expression ${expression}`,
      );
  }
}
