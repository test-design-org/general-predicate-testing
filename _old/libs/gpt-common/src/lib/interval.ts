import { Interval } from 'interval-arithmetic';
import { Expression } from './dtos';

class IsOpen {
  constructor(public lo: boolean, public hi: boolean) {}

  static closed() {
    return new IsOpen(false, false);
  }

  static open() {
    return new IsOpen(true, true);
  }
}

export class MultiInterval {
  private intervals: [IsOpen, Interval][] = [];

  private constructor(intervals: [IsOpen, Interval][]) {
    this.intervals = intervals;
  }

  public DONTUSE_getInterval(): [IsOpen, Interval] {
    return this.intervals[0];
  }

  static simple(interval: Interval, isOpen: IsOpen) {
    return new MultiInterval([[isOpen, interval]]);
  }

  static fromUnaryExpression(
    expression: Expression,
    num: number,
  ): MultiInterval {
    switch (expression) {
      case Expression.LessThan:
        return MultiInterval.simple(
          new Interval(-Infinity, num),
          new IsOpen(true, true),
        );

      case Expression.LessThanOrEqualTo:
        return MultiInterval.simple(
          new Interval(-Infinity, num),
          new IsOpen(true, false),
        );

      case Expression.GreaterThan:
        return MultiInterval.simple(
          new Interval(num, Infinity),
          new IsOpen(true, true),
        );

      case Expression.GreaterThanOrEqualTo:
        return MultiInterval.simple(
          new Interval(num, Infinity),
          new IsOpen(false, true),
        );

      case Expression.EqualTo:
        return MultiInterval.simple(new Interval(num, num), IsOpen.closed());

      // TODO: This should return (-Inf,num) (num, Inf) as a multi-interval
      case Expression.NotEqualTo:
        return MultiInterval.simple(new Interval(num, num), IsOpen.open());

      default:
        throw new Error(
          `Cannot create a unary IntervalDTO from expression ${expression}`,
        );
    }
  }
}
