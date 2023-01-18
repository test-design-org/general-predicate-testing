import React, { useState } from 'react';
import { Graph } from '@testing-repo/gpt-common';
import { Variable } from '@testing-repo/gpt-compiler';
import { generateTestValue } from '@testing-repo/gpt-algorithm';
import './TestCaseTable.scss';

const TestCaseTable = ({
  graph,
  variables,
}: {
  graph: Graph;
  variables: Variable[];
}) => {
  const [showIntervalValues, setShowIntervalValues] = useState(false);

  return (
    <div className="testCaseTable">
      <h2>Generated test cases</h2>

      <label htmlFor="showIntervalValues">
        <input
          type="checkbox"
          checked={showIntervalValues}
          onChange={() => setShowIntervalValues((x) => !x)}
          name="showIntervalValues"
          id="showIntervalValues"
        />
        Show interval values
      </label>

      <br />

      <span>
        <code>*</code> can be any value you like
      </span>

      <table className="testValueTable">
        <thead>
          <tr>
            <th></th>
            {variables.map((x) => (
              <th>{x.name}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {graph.nodes.map((nNuple, index) => (
            <tr>
              <td>T{index + 1}</td>
              {nNuple.list.map((x) => (
                <td>{generateTestValue(x, showIntervalValues)}</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

export default TestCaseTable;
