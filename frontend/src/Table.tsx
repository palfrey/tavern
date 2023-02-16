import React from "react";
import { leaveTable, WS } from "./commands";
import { useUIStore } from "./Store";
import { Videos } from "./Video";

export function Table({ websocket }: { websocket: WS }) {
  const currentPub = useUIStore((s) => s.currentPub());
  const currentTable = useUIStore((s) => s.currentTable());
  console.info("redo table", currentPub, currentTable, websocket);
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
        onClick={(evt) => {
          leaveTable(websocket, currentTable.id);
          evt.preventDefault();
        }}
      >
        Leave table
      </button>
      <Videos websocket={websocket} />
    </div>
  );
}
