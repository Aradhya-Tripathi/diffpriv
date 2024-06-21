import { useState } from "react";
import ConfigForm from "./components/ConfigForm";
import Tables from "./components/Tables";
import "./App.css";
import { Toaster } from "sonner";
import ExecutionWindow from "./components/ExecutionWindow";

function App() {
  const [isConnected, setIsConnected] = useState(false);
  const [isSensitivitySet, setIsSensitivitySet] = useState(false);

  const handleConnection = () => {
    setIsConnected(true);
  };

  const handleSensitivity = () => {
    setIsSensitivitySet(true);
  };

  return (
    <div className="app-container">
      {!isConnected && <ConfigForm onConnect={handleConnection} />}
      {isConnected && !isSensitivitySet && <Tables onSet={handleSensitivity} />}
      {isConnected && isSensitivitySet && <ExecutionWindow />}
      <Toaster />
    </div>
  );
}

export default App;
