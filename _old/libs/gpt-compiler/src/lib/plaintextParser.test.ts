import {
  BoolDTO,
  Expression,
  IntervalDTO,
  MissingVariableDTO,
} from '@testing-repo/gpt-common';
import {
  BoolVariable,
  NumberVariable,
  parseInput,
  parseTestCases,
  parseVariables,
} from './plaintextParser';

describe('parseVariable', () => {
  it('parses a boolVariable', () => {
    const result = parseVariables('varName(bool)')[0];

    expect(result).toBeInstanceOf(BoolVariable);
    expect(result.name).toBe('varName');
  });

  it('parses a numberVariable with int precision', () => {
    const result = parseVariables('x(int)')[0];

    expect(result).toBeInstanceOf(NumberVariable);
    expect(result).toEqual({ name: 'x', precision: 1 });
  });

  it('parses a numberVariable with default precision', () => {
    const result = parseVariables('x(num)')[0];

    expect(result).toBeInstanceOf(NumberVariable);
    expect(result).toEqual({ name: 'x', precision: 0.01 });
  });

  it('parses a numberVariable with float precision', () => {
    const result = parseVariables('x(num,0.23)')[0];

    expect(result).toBeInstanceOf(NumberVariable);
    expect(result).toEqual({ name: 'x', precision: 0.23 });
  });

  it('throws an error on undefined pattern', () => {
    const testCases = [
      'x(num,0.01,)',
      'x(num,qwe)',
      'x(num,)',
      'x(int,)',
      'x(number)',
      'asd',
      'asd()',
      'asd(boolean,)',
      'asd(bool,)',
      'asd(Bool)',
    ];

    for (const testCase of testCases) {
      expect(() => parseVariables(testCase)).toThrow(Error);
    }
  });
});

describe('parseTestCase', () => {
  describe('bool test case', () => {
    it('should parse true', () => {
      const result = parseTestCases([new BoolVariable('x')], 'true')[0];

      expect(result).toBeInstanceOf(BoolDTO);
      const dto = result as BoolDTO;
      expect(dto.boolVal).toBe(true);
      expect(dto.expression).toBe(Expression.BoolTrue);
      expect(dto.isConstant).toBe(false);
    });

    it('should parse false', () => {
      const result = parseTestCases([new BoolVariable('x')], 'false')[0];

      expect(result).toBeInstanceOf(BoolDTO);
      const dto = result as BoolDTO;
      expect(dto.boolVal).toBe(false);
      expect(dto.expression).toBe(Expression.BoolFalse);
      expect(dto.isConstant).toBe(false);
    });

    it('should parse $false', () => {
      const result = parseTestCases([new BoolVariable('x')], '$false')[0];

      expect(result).toBeInstanceOf(BoolDTO);
      const dto = result as BoolDTO;
      expect(dto.boolVal).toBe(false);
      expect(dto.expression).toBe(Expression.BoolFalse);
      expect(dto.isConstant).toBe(true);
    });

    it('should throw on variable mismatch', () => {
      expect(() =>
        parseTestCases([new NumberVariable('x', 1)], 'false'),
      ).toThrowError();
      expect(() =>
        parseTestCases([new NumberVariable('x', 1)], 'true'),
      ).toThrowError();
    });
  });

  describe('unary operator case', () => {
    it('should parse <', () => {
      const result = parseTestCases([new NumberVariable('x', 0.1)], '<30')[0];

      expect(result).toBeInstanceOf(IntervalDTO);
      const dto = result as IntervalDTO;
      expect(dto.expression).toBe(Expression.LessThan);
      expect(dto.interval.lo).toBe(-Infinity);
      expect(dto.interval.hi).toBe(30);
      expect(dto.isOpen).toEqual({ lo: true, hi: true });
      expect(dto.isConstant).toBe(false);
    });

    it('should parse <=', () => {
      const result = parseTestCases([new NumberVariable('x', 0.1)], '<=-30')[0];

      expect(result).toBeInstanceOf(IntervalDTO);
      const dto = result as IntervalDTO;
      expect(dto.expression).toBe(Expression.LessThanOrEqualTo);
      expect(dto.interval.lo).toBe(-Infinity);
      expect(dto.interval.hi).toBe(-30);
      expect(dto.isOpen).toEqual({ lo: true, hi: false });
      expect(dto.isConstant).toBe(false);
    });

    it('should parse >', () => {
      const result = parseTestCases([new NumberVariable('x', 0.1)], '>30')[0];

      expect(result).toBeInstanceOf(IntervalDTO);
      const dto = result as IntervalDTO;
      expect(dto.expression).toBe(Expression.GreaterThan);
      expect(dto.interval.lo).toBe(30);
      expect(dto.interval.hi).toBe(Infinity);
      expect(dto.isOpen).toEqual({ lo: true, hi: true });
      expect(dto.isConstant).toBe(false);
    });

    it('should parse >=', () => {
      const result = parseTestCases([new NumberVariable('x', 0.1)], '>=30')[0];

      expect(result).toBeInstanceOf(IntervalDTO);
      const dto = result as IntervalDTO;
      expect(dto.expression).toBe(Expression.GreaterThanOrEqualTo);
      expect(dto.interval.lo).toBe(30);
      expect(dto.interval.hi).toBe(Infinity);
      expect(dto.isOpen).toEqual({ lo: false, hi: true });
      expect(dto.isConstant).toBe(false);
    });

    it('should parse =', () => {
      const result = parseTestCases([new NumberVariable('x', 0.1)], '=-30')[0];

      expect(result).toBeInstanceOf(IntervalDTO);
      const dto = result as IntervalDTO;
      expect(dto.expression).toBe(Expression.EqualTo);
      expect(dto.interval.lo).toBe(-30);
      expect(dto.interval.hi).toBe(-30);
      expect(dto.isOpen).toEqual({ lo: false, hi: false });
      expect(dto.isConstant).toBe(false);
    });

    it('should parse !=', () => {
      const result = parseTestCases([new NumberVariable('x', 0.1)], '!=30')[0];

      expect(result).toBeInstanceOf(IntervalDTO);
      const dto = result as IntervalDTO;
      expect(dto.expression).toBe(Expression.NotEqualTo);
      expect(dto.interval.lo).toBe(30);
      expect(dto.interval.hi).toBe(30);
      expect(dto.isOpen).toEqual({ lo: false, hi: false });
      expect(dto.isConstant).toBe(false);
    });

    it('should parse constants', () => {
      const result = parseTestCases([new NumberVariable('x', 0.1)], '$<30')[0];

      expect(result).toBeInstanceOf(IntervalDTO);
      const dto = result as IntervalDTO;
      expect(dto.expression).toBe(Expression.LessThan);
      expect(dto.interval.lo).toBe(-Infinity);
      expect(dto.interval.hi).toBe(30);
      expect(dto.isOpen).toEqual({ lo: true, hi: true });
      expect(dto.isConstant).toBe(true);
    });

    it('should parse with whitespace', () => {
      const result = parseTestCases(
        [new NumberVariable('x', 0.1)],
        '$   \t  !=  \t   30',
      )[0];

      expect(result).toBeInstanceOf(IntervalDTO);
      const dto = result as IntervalDTO;
      expect(dto.expression).toBe(Expression.NotEqualTo);
      expect(dto.interval.lo).toBe(30);
      expect(dto.interval.hi).toBe(30);
      expect(dto.isOpen).toEqual({ lo: false, hi: false });
      expect(dto.isConstant).toBe(true);
    });
  });

  describe('interval case', () => {
    it('should parse (-10,22.3]', () => {
      const result = parseTestCases(
        [new NumberVariable('x', 0.1)],
        '(-10,22.3]',
      )[0];

      expect(result).toBeInstanceOf(IntervalDTO);
      const dto = result as IntervalDTO;
      expect(dto.expression).toBe(Expression.Interval);
      expect(dto.interval.lo).toBe(-10);
      expect(dto.interval.hi).toBe(22.3);
      expect(dto.isOpen).toEqual({ lo: true, hi: false });
      expect(dto.isConstant).toBe(false);
    });

    it('should parse [10.12,22)', () => {
      const result = parseTestCases(
        [new NumberVariable('x', 0.1)],
        '[10.12,22)',
      )[0];

      expect(result).toBeInstanceOf(IntervalDTO);
      const dto = result as IntervalDTO;
      expect(dto.expression).toBe(Expression.Interval);
      expect(dto.interval.lo).toBe(10.12);
      expect(dto.interval.hi).toBe(22);
      expect(dto.isOpen).toEqual({ lo: false, hi: true });
      expect(dto.isConstant).toBe(false);
    });

    it('should parse $[10.12,22)', () => {
      const result = parseTestCases(
        [new NumberVariable('x', 0.1)],
        '$[-10.12,0.1)',
      )[0];

      expect(result).toBeInstanceOf(IntervalDTO);
      const dto = result as IntervalDTO;
      expect(dto.expression).toBe(Expression.Interval);
      expect(dto.interval.lo).toBe(-10.12);
      expect(dto.interval.hi).toBe(0.1);
      expect(dto.isOpen).toEqual({ lo: false, hi: true });
      expect(dto.isConstant).toBe(true);
    });

    it('should parse whitespaces', () => {
      const result = parseTestCases(
        [new NumberVariable('x', 0.1)],
        '$    [  -10.12   ,   0.1   )',
      )[0];

      expect(result).toBeInstanceOf(IntervalDTO);
      const dto = result as IntervalDTO;
      expect(dto.expression).toBe(Expression.Interval);
      expect(dto.interval.lo).toBe(-10.12);
      expect(dto.interval.hi).toBe(0.1);
      expect(dto.isOpen).toEqual({ lo: false, hi: true });
      expect(dto.isConstant).toBe(true);
    });
  });

  describe('wrong input', () => {
    it('should throw error on wrong input', () => {
      expect(() =>
        parseTestCases([new BoolVariable('x')], 'adsqwe'),
      ).toThrowError();
      expect(() =>
        parseTestCases([new NumberVariable('x', 1)], '>='),
      ).toThrowError();
      expect(() =>
        parseTestCases([new NumberVariable('x', 1)], '>=asd'),
      ).toThrowError();
      expect(() =>
        parseTestCases([new NumberVariable('x', 1)], '>=56asd'),
      ).toThrowError();
      expect(() =>
        parseTestCases([new NumberVariable('x', 1)], '>=56asd'),
      ).toThrowError();
      expect(() =>
        parseTestCases([new NumberVariable('x', 1)], '56>=56'),
      ).toThrowError();
      expect(() =>
        parseTestCases([new NumberVariable('x', 1)], '56='),
      ).toThrowError();
      expect(() =>
        parseTestCases([new NumberVariable('x', 1)], '56'),
      ).toThrowError();
    });
  });
});

describe('parseInput', () => {
  it('should parse the input correctly', () => {
    const testInput = `
      // This is a comment, this should be ignored

      VIP(bool); price(int); second_hand_price(num,0.2)
      true   ; <50   ;   *  

      // A comment can be anywhere
      *; >30; <=60

      // Check constant values
        $  false  ;   $  < 20;   $  (  -10.3 ,  45 ] 
      
      `;

    const result = parseInput(testInput);

    expect(result.length).toBe(2);

    const firstCase = result[1][0];

    expect(firstCase[0]).toBeInstanceOf(BoolDTO);
    expect(firstCase[0]).toEqual({
      expression: Expression.BoolTrue,
      boolVal: true,
      isConstant: false,
    });

    expect(firstCase[1]).toBeInstanceOf(IntervalDTO);
    const interval1 = firstCase[1] as IntervalDTO;
    expect(interval1.expression).toBe(Expression.LessThan);
    expect(interval1.interval.lo).toBe(-Infinity);
    expect(interval1.interval.hi).toBe(50);
    expect(interval1.isOpen).toEqual({ lo: true, hi: true });
    expect(interval1.isConstant).toBe(false);

    expect(firstCase[2]).toBeInstanceOf(MissingVariableDTO);

    const secondCase = result[1][1];
    expect(secondCase[0]).toBeInstanceOf(MissingVariableDTO);

    expect(secondCase[1]).toBeInstanceOf(IntervalDTO);
    const interval2 = secondCase[1] as IntervalDTO;
    expect(interval2.expression).toBe(Expression.GreaterThan);
    expect(interval2.interval.lo).toBe(30);
    expect(interval2.interval.hi).toBe(Infinity);
    expect(interval2.isOpen).toEqual({ lo: true, hi: true });
    expect(interval2.isConstant).toBe(false);

    expect(secondCase[2]).toBeInstanceOf(IntervalDTO);
    const interval3 = secondCase[2] as IntervalDTO;
    expect(interval3.expression).toBe(Expression.LessThanOrEqualTo);
    expect(interval3.interval.lo).toBe(-Infinity);
    expect(interval3.interval.hi).toBe(60);
    expect(interval3.isOpen).toEqual({ lo: true, hi: false });
    expect(interval3.isConstant).toBe(false);

    const thirdCase = result[1][2];
    expect(thirdCase[0]).toBeInstanceOf(BoolDTO);
    expect(thirdCase[0]).toEqual({
      expression: Expression.BoolFalse,
      boolVal: false,
      isConstant: true,
    });

    expect(thirdCase[1]).toBeInstanceOf(IntervalDTO);
    const interval4 = thirdCase[1] as IntervalDTO;
    expect(interval4.expression).toBe(Expression.LessThan);
    expect(interval4.interval.lo).toBe(-Infinity);
    expect(interval4.interval.hi).toBe(20);
    expect(interval4.isOpen).toEqual({ lo: true, hi: true });
    expect(interval4.isConstant).toBe(true);

    expect(thirdCase[2]).toBeInstanceOf(IntervalDTO);
    const interval5 = thirdCase[2] as IntervalDTO;
    expect(interval5.expression).toBe(Expression.Interval);
    expect(interval5.interval.lo).toBe(-10.3);
    expect(interval5.interval.hi).toBe(45);
    expect(interval5.isOpen).toEqual({ lo: true, hi: false });
    expect(interval5.isConstant).toBe(true);
  });
});

export {};
