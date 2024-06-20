import { useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";

const ConfigForm = () => {
  const [configPath, setConfigPath] = useState("");
  const [databaseType, setDatabaseType] = useState("sqlite");

  async function configure() {
    if (configPath && databaseType) {
      await invoke("configure", {
        configPath: configPath,
        databaseType: databaseType,
      });
      setConfigPath("");
      setDatabaseType("");
    }
  }

  const handleSubmit = (e) => {
    e.preventDefault();
    configure();
  };

  return (
    <div className="config-form-container">
      <form onSubmit={handleSubmit}>
        <div className="form-group">
          <label htmlFor="configPath">Configuration Path:</label>
          <input
            type="text"
            id="configPath"
            value={configPath}
            onChange={(e) => setConfigPath(e.target.value)}
            required
          />
        </div>
        <div className="form-group">
          <label htmlFor="dbType">Database Type:</label>
          <select
            id="dbType"
            value={databaseType}
            onChange={(e) => setDatabaseType(e.target.value)}
          >
            <option value="sqlite">SQLite</option>
            <option value="mysql">MySQL</option>
          </select>
        </div>
        <button type="submit">Submit</button>
      </form>
    </div>
  );
};

export default ConfigForm;
