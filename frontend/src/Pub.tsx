import { useState } from "react";
import {
  createTable,
  deleteTable,
  joinTable,
  leavePub,
  listTables,
} from "./commands";
import { useUIStore } from "./Store";
import { useWebsocket } from "./Websocket";

export function Pub() {
  const [tableName, setTableName] = useState("");
  const currentPub = useUIStore((s) => s.currentPub())!;
  const tables = useUIStore((s) => s.tables);
  const websocket = useWebsocket();
  return (
    <div>
      {" "}
      <h1>{currentPub.name}</h1>
      <br />
      <button
        className="btn btn-danger"
        onClick={() => leavePub(websocket, currentPub.id)}
      >
        Leave pub
      </button>
      <br />
      <input
        type="button"
        className="btn btn-secondary"
        value="Update table list"
        onClick={() => listTables(websocket, currentPub.id)}
      />
      <div>Tables</div>
      <ul>
        {tables.map((table) => (
          <li key={table.id}>
            {table.name}
            <span>&nbsp;</span>
            <button
              className="btn btn-primary"
              onClick={() => joinTable(websocket, table.id)}
            >
              Join
            </button>
            <span>&nbsp;</span>
            {table.persons.length == 0 && (
              <button
                className="btn btn-danger"
                onClick={() => deleteTable(websocket, table.id)}
              >
                Delete
              </button>
            )}
          </li>
        ))}
      </ul>
      <form>
        <div className="form-group">
          <label htmlFor="tableName">New table</label>
          <input
            type="text"
            className="form-control"
            id="tableName"
            placeholder="Enter table name"
            value={tableName}
            onChange={(evt) => setTableName(evt.target.value)}
          />
        </div>
        <button
          type="button"
          className="btn btn-primary"
          onClick={() => createTable(websocket, currentPub.id, tableName)}
        >
          Create table
        </button>
      </form>
    </div>
  );
}
