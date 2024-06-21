import { invoke } from "@tauri-apps/api";
import { useEffect, useState } from "react";

const Tables = ({ onSet }) => {
  const [tables, setTables] = useState([]);
  const [inputValues, setInputValues] = useState({});

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

  const handleSensitivityInput = async () => {
    const convertedValues = {};
    for (const table in inputValues) {
      convertedValues[table] = {};
      for (const column in inputValues[table]) {
        const value = inputValues[table][column];
        convertedValues[table][column] = value === "" ? 0.0 : parseFloat(value);
      }
    }
    console.log(convertedValues);
    await invoke("set_sensitivities", { sensitivities: convertedValues });
    onSet();
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
                    type="number"
                    placeholder={column.name}
                    value={inputValues[table.name]?.[column.name] || ""}
                    onChange={(e) =>
                      handleInputChange(table.name, column.name, e.target.value)
                    }
                  />
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
      <button onClick={handleSensitivityInput}>Set Sensitivity</button>
    </>
  );
};

export default Tables;
