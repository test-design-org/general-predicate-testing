import { IInput } from './dtos';
import { zip } from 'fp-ts/Array';
import { Eq } from 'fp-ts/lib/Eq';
import { array } from 'fp-ts';
import { v4 as uuidv4 } from 'uuid';

export class NTuple {
  readonly list: IInput[];
  readonly id = uuidv4();

  constructor(list: IInput[]) {
    this.list = list;
  }

  intersectsWith(other: NTuple): boolean {
    if (this.list.length !== other.list.length) return false;

    return zip(this.list, other.list).every(([x, y]) => x.intersectsWith(y));
  }

  intersect(other: NTuple): NTuple {
    if (!this.intersectsWith(other))
      throw new Error('Cannot intersect NTuples!');

    const intersectedInputs = zip(this.list, other.list).map(([x, y]) =>
      x.intersect(y),
    );

    return new NTuple(intersectedInputs);
  }

  toString(): string {
    return this.list.map((x) => x.toString()).join('_');
  }
}

// eslint-disable-next-line @typescript-eslint/no-namespace
export namespace NTuple {
  export const Eq: Eq<NTuple> = {
    equals(x: NTuple, y: NTuple): boolean {
      if (x === y) return true;

      return array.getEq(IInput.Eq).equals(x.list, y.list);
    },
  };
}
