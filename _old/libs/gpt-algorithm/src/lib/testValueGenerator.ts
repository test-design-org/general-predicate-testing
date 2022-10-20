import {
  BoolDTO,
  IInput,
  IntervalDTO,
  MissingVariableDTO,
} from '@testing-repo/gpt-common';

export function generateTestValue(
  input: IInput,
  showIntervalValues: boolean,
): number | string {
  if (input instanceof MissingVariableDTO) return '*';

  if (input instanceof BoolDTO) return input.boolVal ? 'true' : 'false';

  if (showIntervalValues) return (input as IntervalDTO).toString();

  const interval = (input as IntervalDTO).interval;
  if (interval.lo === -Infinity) return interval.hi;

  if (interval.hi === Infinity) return interval.lo;

  return interval.lo;
}
