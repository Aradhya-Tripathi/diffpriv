import { invoke } from "@tauri-apps/api";
import { useState } from "react";

const ExecutionWindow = () => {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState([]);

  const handleInputChange = (e) => {
    setInput(e.target.value);
  };

  const handleExecute = async () => {
    // Mock execution of SQL query
    await invoke("execute_sql", { query: input });
    const result = `Executed: ${input}`;
    setOutput([...output, result]);
    setInput("");
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
          onKeyDown={(e) => e.key === "Enter" && handleExecute()}
          className="input-field"
          placeholder="Enter SQL query here..."
        />
      </div>
    </div>
  );
};

export default ExecutionWindow;
