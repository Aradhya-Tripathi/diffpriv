import { invoke } from "@tauri-apps/api";
import { useState, useEffect } from "react";
import { register } from "@tauri-apps/api/globalShortcut";
import ConfigForm from "./components/ConfigForm";
import Tables from "./components/Tables";
import "./styles/App.css";
import { Toaster } from "sonner";
import ExecutionWindow from "./components/ExecutionWindow";

function App() {
  const [isConnected, setIsConnected] = useState(false);
  const [isSensitivitySet, setIsSensitivitySet] = useState(false);

  useEffect(() => {
    async function registerShortcut() {
      await register("CommandOrControl+R", async () => {
        await invoke("reset_sensitivities");
        setIsSensitivitySet(false);
      });
      await register("CommandOrControl+Shift+R", async () => {
        await invoke("reset_connection");
        setIsSensitivitySet(false);
        setIsConnected(false);
      });
    }
    registerShortcut();
  }, []);

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
