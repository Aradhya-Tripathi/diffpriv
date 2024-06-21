import { invoke } from "@tauri-apps/api";
import { useState } from "react";

const ExecutionWindow = () => {
  const [input, setInput] = useState("");
  const [budget, setbudget] = useState("");
  const [output, setOutput] = useState([]);

  const handleInputChange = (e) => {
    setInput(e.target.value);
  };

  const handleFloatChange = (e) => {
    setbudget(e.target.value);
  };

  const handleExecute = async () => {
    if (!budget) {
      toast.error("Provide the budget for the query!");
      return;
    }

    // Mock execution of SQL query
    await invoke("execute_sql", {
      query: input,
      budget: parseFloat(budget),
    });
    const result = `Executed: ${input} with value: ${budget}`;
    setOutput([...output, result]);
    setInput("");
    setbudget("");
  };

  return (
    <div className="exc-window">
      <div className="output-window">
        {output.map((line, index) => (
          <div key={index} className="output-line">
            {line}
          </div>
        ))}
      </div>
      <div className="input-window">
        <input
          type="text"
          value={input}
          onChange={handleInputChange}
          className="input-field first"
          placeholder="Enter SQL..."
        />
        <div className="button-and-input">
          <input
            type="text"
            value={budget}
            onChange={handleFloatChange}
            className="input-field second"
            placeholder="Enter budget..."
          />
        </div>
        <button onClick={handleExecute} className="execute-button">
          Execute
        </button>
      </div>
    </div>
  );
};

export default ExecutionWindow;
