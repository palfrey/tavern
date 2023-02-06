import React from "react";
import { leaveTable } from "./commands";
import { useUIStore } from "./Store";
import { Videos } from "./Video";
import { useWebsocket } from "./Websocket";

export function Table() {
  const currentPub = useUIStore((s) => s.currentPub());
  const currentTable = useUIStore((s) => s.currentTable());
  const websocket = useWebsocket();
  if (currentPub === null || currentTable == null) {
    // We'll nav away from here soon...
    return <React.Fragment />;
  }
  return (
    <div>
      <h1>
        {currentPub.name}: {currentTable.name}
      </h1>
      <br />
      <button
        className="btn btn-danger"
        onClick={() => leaveTable(websocket, currentTable.id)}
      >
        Leave table
      </button>
      <Videos />
    </div>
  );
}
