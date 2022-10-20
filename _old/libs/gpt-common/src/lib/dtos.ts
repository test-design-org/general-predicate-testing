import { Eq } from 'fp-ts/lib/Eq';
import IOps, { Interval } from 'interval-arithmetic';

export enum Expression {
  LessThan,
  LessThanOrEqualTo,
  GreaterThan,
  GreaterThanOrEqualTo,
  EqualTo,
  NotEqualTo,
  BoolTrue,
  BoolFalse,
  Interval,
  MissingVariable,
}

export interface IInput {
  expression: Expression;
  isConstant: boolean;

  intersectsWith(other: IInput): boolean;
  intersect(other: IInput): IInput;
  withPrecision(newPrecision: number): IInput;
  toString(): string;
}

export class BoolDTO implements IInput {
  expression: Expression;
  boolVal: boolean;
  isConstant: boolean;

  constructor(expression: Expression, boolVal: boolean, isConstant = false) {
    this.expression = expression;
    this.boolVal = boolVal;
    this.isConstant = isConstant;
  }

  intersectsWith(other: IInput): boolean {
    if (other instanceof MissingVariableDTO) return true;
    if (other instanceof BoolDTO) return other.boolVal === this.boolVal;

    return false;
  }

  intersect(other: IInput): IInput {
    if (!this.intersectsWith(other))
      throw new Error(`Cannot intersect types ${this} ${other}`);

    // This may seem redundant, but might be useful in the future
    if (other instanceof MissingVariableDTO)
      return new BoolDTO(this.expression, this.boolVal);

    return new BoolDTO(this.expression, this.boolVal);
  }

  withPrecision(_newPrecision: number): BoolDTO {
    return this;
  }

  toString(): string {
    return `${this.boolVal}`;
  }

  equals(other: BoolDTO): boolean {
    if (this === other) return true;

    return (
      this.expression === other.expression && this.boolVal === other.boolVal
    );
  }
}

export class MissingVariableDTO implements IInput {
  expression = Expression.MissingVariable;
  isConstant = false;

  intersectsWith(_other: IInput): boolean {
    return true;
  }
  intersect(other: IInput): IInput {
    return other;
  }

  withPrecision(_newPrecision: number): MissingVariableDTO {
    return this;
  }

  toString(): string {
    return `*`;
  }
  equals(other: MissingVariableDTO) {
    return true;
  }
}

export interface IsOpen {
  hi: boolean;
  lo: boolean;
}

export class IntervalDTO implements IInput {
  expression: Expression;
  isConstant: boolean;
  interval: Interval;
  isOpen: IsOpen;
  precision: number;

  constructor(
    expression: Expression,
    interval: Interval,
    precision: number,
    isOpen: IsOpen = { hi: false, lo: false },
    isConstant = false,
  ) {
    this.expression = expression;
    this.interval = interval;
    this.precision = precision;
    this.isOpen = isOpen;
    this.isConstant = isConstant;
  }

  intersectsWith(other: IInput): boolean {
    if (other instanceof MissingVariableDTO) return true;
    if (other instanceof IntervalDTO)
      return IOps.intervalsOverlap(this.interval, other.interval);

    return false;
  }

  intersect(other: IInput): IInput {
    if (!this.intersectsWith(other))
      throw new Error(`Cannot intersect types ${this} ${other}`);

    if (other instanceof MissingVariableDTO)
      return new IntervalDTO(this.expression, this.interval, this.precision);

    const that = other as IntervalDTO;
    return new IntervalDTO(
      this.expression,
      IOps.intersection(this.interval, that.interval),
      this.precision,
    );
  }

  withPrecision(newPrecision: number): IntervalDTO {
    return new IntervalDTO(
      this.expression,
      this.interval,
      newPrecision,
      this.isOpen,
      this.isConstant,
    );
  }

  toString(): string {
    if (this.interval.lo === this.interval.hi) return `${this.interval.lo}`;

    const left =
      this.interval.lo === -Infinity ? '-∞' : this.interval.lo.toString();
    const right =
      this.interval.hi === Infinity ? '∞' : this.interval.hi.toString();

    const leftBrace = this.isOpen.lo ? '(' : '[';
    const rightBrace = this.isOpen.hi ? ')' : ']';

    return `${leftBrace}${left},${right}${rightBrace}`;
  }

  equals(other: IntervalDTO): boolean {
    if (this === other) return true;

    return (
      this.expression === other.expression &&
      IOps.equal(this.interval, other.interval) &&
      this.precision === other.precision &&
      this.isOpen.hi === other.isOpen.hi &&
      this.isOpen.lo === other.isOpen.lo
    );
  }
}

// eslint-disable-next-line @typescript-eslint/no-namespace
export namespace IInput {
  export const Eq: Eq<IInput> = {
    equals(x: IInput, y: IInput) {
      if (x instanceof BoolDTO && y instanceof BoolDTO) return x.equals(y);

      if (x instanceof IntervalDTO && y instanceof IntervalDTO)
        return x.equals(y);

      if (x instanceof MissingVariableDTO && y instanceof MissingVariableDTO)
        return x.equals(y);

      return false;
    },
  };
}
