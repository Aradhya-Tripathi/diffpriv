import { useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { toast } from "sonner";
import "../styles/Config.css";

const ConfigForm = ({ onConnect }) => {
  const [databasePath, setDatabasePath] = useState("");

  const configure = async () => {
    if (databasePath.trim()) {
      try {
        const msg = await invoke("connect", { databasePath });
        setDatabasePath("");
        toast.success(msg, { duration: 2000 });
        onConnect();
      } catch (err) {
        toast.error(err, { duration: 2000 });
      }
    } else {
      toast.error("Database path cannot be empty", { duration: 2000 });
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
          <label htmlFor="configPath">Database Path/URI</label>
          <input
            type="text"
            id="configPath"
            value={databasePath}
            onChange={(e) => setDatabasePath(e.target.value)}
            placeholder="Enter Path/URI"
            required
          />
        </div>
        <button type="submit">Submit</button>
      </form>
    </div>
  );
};

export default ConfigForm;
