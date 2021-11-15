import {
  DetailsList,
  IColumn,
  SelectionMode,
  PrimaryButton,
} from "@fluentui/react";
import * as React from "react";
import { Variable } from "../models/Types";
import DeleteButton from "./DeleteButton";
import { useBoolean } from "@fluentui/react-hooks";
import { AddVariableDialog } from "./AddVariableDialog";

/**
 * This is the Module header on the top
 * @returns Module name two lines
 */

const columns: IColumn[] = [
  {
    key: "name",
    name: "Name",
    fieldName: "name",
    minWidth: 210,
    maxWidth: 350,
    data: "string",
    isRowHeader: true,
    isResizable: true,
    isSorted: true,
    isSortedDescending: false,
    sortAscendingAriaLabel: "Sorted A to Z",
    sortDescendingAriaLabel: "Sorted Z to A",
    isPadded: true,
  },
  {
    key: "type",
    name: "Type",
    fieldName: "type",
    minWidth: 210,
    maxWidth: 350,
    data: "string",
    isRowHeader: true,
    isResizable: true,
    isSorted: true,
    isSortedDescending: false,
    sortAscendingAriaLabel: "Sorted A to Z",
    sortDescendingAriaLabel: "Sorted Z to A",
    isPadded: true,
  },
  {
    key: "precision",
    name: "Precision",
    fieldName: "precision",
    minWidth: 210,
    maxWidth: 350,
    data: "string",
    isRowHeader: true,
    isResizable: true,
    isSorted: true,
    isSortedDescending: false,
    sortAscendingAriaLabel: "Sorted A to Z",
    sortDescendingAriaLabel: "Sorted Z to A",
    isPadded: true,
  },
  {
    key: "delete",
    name: "",
    fieldName: "delete",
    minWidth: 210,
    maxWidth: 350,
    data: "string",
    isRowHeader: true,
    isResizable: true,
    isSorted: true,
    isSortedDescending: false,
    sortAscendingAriaLabel: "Sorted A to Z",
    sortDescendingAriaLabel: "Sorted Z to A",
    isPadded: true,
  },
];

type VariableSetterProps = {
  variables: Variable[];
  setVariables: React.Dispatch<React.SetStateAction<Variable[]>>;
};

const VariableSetter: React.FunctionComponent<VariableSetterProps> = ({
  variables,
  setVariables,
}: VariableSetterProps) => {
  const items: any = variables.map((variable: Variable) => ({
    key: variable.name,
    name: variable.name,
    type: variable.type,
    precision: variable.precision !== undefined ? variable.precision : "-",
    delete: (
      <DeleteButton setVariables={setVariables} variableName={variable.name} />
    ),
  }));

  const [hideDialog, { toggle: toggleHideDialog }] = useBoolean(true);

  return (
    <>
      <PrimaryButton onClick={toggleHideDialog} text="Add variable" />
      <AddVariableDialog
        hideDialog={hideDialog}
        toggleHideDialog={toggleHideDialog}
        variables={variables}
        setVariables={setVariables}
      />
      <DetailsList
        items={items}
        columns={columns}
        selectionMode={SelectionMode.none}
        enableUpdateAnimations={true}
      ></DetailsList>
    </>
  );
};

export default VariableSetter;
