import { invoke } from "@tauri-apps/api";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import "../styles/Table.css";

const Tables = ({ onSet }) => {
  const [tables, setTables] = useState([]);
  const [inputValues, setInputValues] = useState({});
  const [tableBudgets, setTableBudgets] = useState({});

  const get_tables = async () => {
    try {
      let tables = await invoke("get_tables");
      console.log(tables);
      setTables(tables);

      // Initialize input values state
      const initialInputValues = {};
      tables.forEach((table) => {
        initialInputValues[table.name] = {};
        table.columns.forEach((column) => {
          initialInputValues[table.name][column.name] = "";
        });
      });
      setInputValues(initialInputValues);
    } catch (err) {
      console.log(err);
    }
  };

  useEffect(() => {
    get_tables();
  }, []);

  const handleInputChange = (tableName, columnName, value) => {
    setInputValues((prevValues) => ({
      ...prevValues,
      [tableName]: {
        ...prevValues[tableName],
        [columnName]: value,
      },
    }));
  };

  const handleBudgetChange = (tableName, budget) => {
    setTableBudgets((prevBudget) => ({
      ...prevBudget,
      [tableName]: budget,
    }));
  };

  const handleSensitivityInput = async () => {
    const convertedValues = {};
    const convertedBudgetValues = {};
    for (const table in inputValues) {
      convertedValues[table] = {};
      for (const column in inputValues[table]) {
        const value = inputValues[table][column];
        convertedValues[table][column] = value === "" ? 0.0 : parseFloat(value);
      }
    }
    for (let table in tableBudgets) {
      convertedBudgetValues[table] =
        tableBudgets[table] === "" ? 0.0 : parseFloat(tableBudgets[table]);
    }
    try {
      let sensitivity_msg = await invoke("set_sensitivities", {
        sensitivities: convertedValues,
      });
      toast.success(sensitivity_msg);
      let budget_message = await invoke("set_budgets", {
        budgets: convertedBudgetValues,
      });
      toast.success(budget_message);
      onSet();
    } catch (err) {
      toast.error(err, { duration: 2000 });
    }
  };

  return (
    <>
      <div className="tables-container">
        {tables.map((table, index) => (
          <div key={index} className="table-card">
            <h2 className="table-name">{table.name}</h2>
            <div className="table-columns">
              {table.columns.map((column, colIndex) => (
                <div key={colIndex} className="table-column">
                  <input
                    type="text"
                    placeholder={column.name}
                    value={inputValues[table.name]?.[column.name] || ""}
                    onChange={(e) =>
                      handleInputChange(table.name, column.name, e.target.value)
                    }
                  />
                </div>
              ))}

              <div className="table-column">
                <input
                  type="text"
                  placeholder="Allowed Budget"
                  value={tableBudgets[table.name] || ""}
                  onChange={(e) =>
                    handleBudgetChange(table.name, e.target.value)
                  }
                />
              </div>
            </div>
          </div>
        ))}
      </div>
      <button onClick={handleSensitivityInput}>Set Sensitivity</button>
    </>
  );
};

export default Tables;
