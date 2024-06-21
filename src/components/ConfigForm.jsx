import { useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";

const ConfigForm = ({ onConnect }) => {
  const [databasePath, setDatabasePath] = useState("");

  const configure = async () => {
    if (databasePath) {
      try {
        const msg = await invoke("connect", { databasePath });
        setDatabasePath("");
        onConnect();
      } catch (err) {
        console.log(err);
      }
    }
  };

  const handleSubmit = (e) => {
    e.preventDefault();
    configure();
  };

  return (
    <div className="config-form-container" id="config-form">
      <form onSubmit={handleSubmit}>
        <div className="form-group">
          <input
            type="text"
            id="configPath"
            value={databasePath}
            onChange={(e) => setDatabasePath(e.target.value)}
            placeholder="Path/URI"
            required
          />
        </div>
        <button type="submit">Submit</button>
      </form>
    </div>
  );
};

export default ConfigForm;
