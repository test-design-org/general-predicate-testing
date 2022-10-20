import { Expression, MultiInterval } from '@testing-repo/gpt-common';
import { EqOp, FeatureNode } from './AST';
import {
  convertBoolCondition,
  resolveBoolCondition,
  traverseASTtoIR,
} from './ASTtoIR';
import { GPT } from './IR';

describe('resolveBoolCondition', () => {
  const table: {
    eqOp: EqOp;
    boolVal: boolean;
    result: boolean;
  }[] = [
    {
      eqOp: '=',
      boolVal: true,
      result: true,
    },
    {
      eqOp: '=',
      boolVal: false,
      result: false,
    },
    {
      eqOp: '!=',
      boolVal: true,
      result: false,
    },
    {
      eqOp: '!=',
      boolVal: false,
      result: true,
    },
  ];

  it.each(table)(
    'gives back the correct expression',
    ({ eqOp, boolVal, result }) => {
      expect(resolveBoolCondition(eqOp, boolVal)).toStrictEqual(result);
    },
  );
});

// describe('convertBoolCondition', () => {
//   it('gives back the expected results', () => {
//     convertBoolCondition();
//   });
// });

describe('traverseASTtoIR', () => {
  it('should work as expected', () => {
    const input: FeatureNode = {
      type: 'feature',
      variables: [
        { type: 'var', varName: 'VIP', varType: { type: 'bool' } },
        {
          type: 'var',
          varName: 'price',
          varType: { type: 'number', precision: 0.01 },
        },
        {
          type: 'var',
          varName: 'second_hand_price',
          varType: { type: 'number', precision: 0.01 },
        },
      ],
      ifStatements: [
        {
          type: 'if',
          conditions: {
            type: 'conditions',
            conditions: [
              { type: 'bool', varName: 'VIP', eqOp: '=', boolVal: true },
              {
                type: 'binary',
                varName: 'price',
                constantPosition: 'rhs',
                constant: 50,
                binaryOp: '<',
              },
            ],
          },
          body: [
            {
              type: 'if',
              conditions: {
                type: 'conditions',
                conditions: [
                  {
                    type: 'binary',
                    varName: 'second_hand_price',
                    constantPosition: 'rhs',
                    constant: 2,
                    binaryOp: '=',
                  },
                ],
              },
              elseIf: [],
            },
          ],
          elseIf: [],
        },
        {
          type: 'if',
          conditions: {
            type: 'conditions',
            conditions: [
              { type: 'bool', varName: 'VIP', eqOp: '=', boolVal: false },
              {
                type: 'binary',
                varName: 'price',
                constantPosition: 'rhs',
                constant: 50,
                binaryOp: '>=',
              },
            ],
          },
          elseIf: [],
        },
        {
          type: 'if',
          conditions: {
            type: 'conditions',
            conditions: [
              { type: 'bool', varName: 'VIP', eqOp: '=', boolVal: true },
              {
                type: 'binary',
                varName: 'price',
                constantPosition: 'rhs',
                constant: 50,
                binaryOp: '>=',
              },
            ],
          },
          elseIf: [],
        },
        {
          type: 'if',
          conditions: {
            type: 'conditions',
            conditions: [
              {
                type: 'binary',
                varName: 'price',
                constantPosition: 'rhs',
                constant: 30,
                binaryOp: '>',
              },
              {
                type: 'binary',
                varName: 'second_hand_price',
                constantPosition: 'rhs',
                constant: 60,
                binaryOp: '>',
              },
            ],
          },
          elseIf: [],
        },
      ],
      features: [],
    };

    const expected: [GPT.Variable[], GPT.Predicate[]] = [
      [
        { varName: 'VIP', type: { type: 'bool' } },
        { varName: 'price', type: { type: 'number', precision: 0.01 } },
        {
          varName: 'second_hand_price',
          type: { type: 'number', precision: 0.01 },
        },
      ],
      [
        [
          {
            type: 'bool',
            varName: 'VIP',
            shouldEqualTo: true,
          },
          {
            type: 'interval',
            varName: 'price',
            expression: Expression.LessThan,
            interval: MultiInterval.fromUnaryExpression(
              Expression.LessThan,
              50,
            ),
          },
          {
            type: 'interval',
            varName: 'second_hand_price',
            expression: Expression.EqualTo,
            interval: MultiInterval.fromUnaryExpression(Expression.EqualTo, 2),
          },
        ],
        [
          {
            type: 'bool',
            varName: 'VIP',
            shouldEqualTo: false,
          },
          {
            type: 'interval',
            varName: 'price',
            expression: Expression.GreaterThanOrEqualTo,
            interval: MultiInterval.fromUnaryExpression(
              Expression.GreaterThanOrEqualTo,
              50,
            ),
          },
        ],
        [
          {
            type: 'bool',
            varName: 'VIP',
            shouldEqualTo: true,
          },
          {
            type: 'interval',
            varName: 'price',
            expression: Expression.GreaterThanOrEqualTo,
            interval: MultiInterval.fromUnaryExpression(
              Expression.GreaterThanOrEqualTo,
              50,
            ),
          },
        ],
        [
          {
            type: 'interval',
            varName: 'price',
            expression: Expression.GreaterThan,
            interval: MultiInterval.fromUnaryExpression(
              Expression.GreaterThan,
              30,
            ),
          },
          {
            type: 'interval',
            varName: 'second_hand_price',
            expression: Expression.GreaterThan,
            interval: MultiInterval.fromUnaryExpression(
              Expression.GreaterThan,
              60,
            ),
          },
        ],
      ],
    ];
    expect(traverseASTtoIR(input)).toStrictEqual(expected);
  });
});

export {};
