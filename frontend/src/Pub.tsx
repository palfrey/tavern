import React from "react";
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
  const currentPub = useUIStore((s) => s.currentPub());
  const tables = useUIStore((s) => s.tables);
  const websocket = useWebsocket();
  if (currentPub === null) {
    // We'll nav away from here soon...
    return <React.Fragment />;
  }
  return (
    <div>
      {" "}
      <h1 id="currentPubName">{currentPub.name}</h1>
      <br />
      <button
        className="btn btn-danger"
        onClick={(evt) => {
          leavePub(websocket, currentPub.id);
          evt.preventDefault();
        }}
      >
        Leave pub
      </button>
      <br />
      <input
        type="button"
        className="btn btn-secondary"
        value="Update table list"
        onClick={(evt) => {
          listTables(websocket, currentPub.id);
          evt.preventDefault();
        }}
      />
      <div>Tables</div>
      <ul>
        {tables.map((table) => (
          <li key={table.id} className="tableItem">
            {table.name}
            <span>&nbsp;</span>
            <button
              className="btn btn-primary"
              onClick={(evt) => {
                joinTable(websocket, table.id);
                evt.preventDefault();
              }}
            >
              Join
            </button>
            <span>&nbsp;</span>
            {table.persons.length == 0 && (
              <button
                className="btn btn-danger"
                onClick={(evt) => {
                  deleteTable(websocket, table.id);
                  evt.preventDefault();
                }}
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
            onChange={(evt) => {
              setTableName(evt.target.value);
              evt.preventDefault();
            }}
          />
        </div>
        <button
          id="createTable"
          type="button"
          className="btn btn-primary"
          onClick={(evt) => {
            createTable(websocket, currentPub.id, tableName);
            evt.preventDefault();
          }}
        >
          Create table
        </button>
      </form>
    </div>
  );
}
