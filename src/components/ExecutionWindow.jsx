import { useState } from "react";
import { invoke } from "@tauri-apps/api";
import { toast } from "sonner";
import "../styles/Execution.css";

const ExecutionWindow = () => {
  const [input, setInput] = useState("");
  const [budget, setBudget] = useState("");
  const [output, setOutput] = useState([]);

  const handleInputChange = (e) => {
    setInput(e.target.value);
  };

  const handleFloatChange = (e) => {
    setBudget(e.target.value);
  };

  const handleExecute = async () => {
    if (!budget) {
      toast.error("Provide the budget for the query!", { duration: 2000 });
      return;
    }

    try {
      let result = await invoke("execute_sql", {
        query: input,
        budget: parseFloat(budget),
      });

      const newOutput = `${input}\n> ${JSON.stringify(result, null)}`;
      setOutput([...output, newOutput]);
      setInput("");
      setBudget("");
    } catch (err) {
      toast.error(err.message, { duration: 2000 });
    }
  };

  return (
    <div className="exc-window">
      <div className="output-window">
        {output.map((line, index) => (
          <pre key={index} className="output-line">
            {line}
          </pre>
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
