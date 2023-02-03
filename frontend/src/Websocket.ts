import { useEffect, useState } from "react";
import { Person, Pub, Table } from "./Data";
import { useUIStore } from "./Store";

export function associate(websocket: WebSocket, accountId: number) {
  console.debug("associating", accountId);
  websocket.send(
    JSON.stringify({ command: "associate", data: { id: accountId } })
  );
}

interface PubsMessage {
  kind: "Pubs";
  list: Pub[];
}

interface TablesMessage {
  kind: "Tables";
  list: Table[];
}

interface PongMessage {
  kind: "Pong";
}

interface PubMessage {
  kind: "Pub";
  data: Pub;
}

interface TableMessage {
  kind: "Table";
  data: Table;
}

interface PersonMessage {
  kind: "Person";
  data: Person;
}

interface DataMessage {
  kind: "Data";
  author: string;
  content: string;
}

type SocketMessage =
  | PubsMessage
  | TablesMessage
  | PongMessage
  | PubMessage
  | TableMessage
  | PersonMessage
  | DataMessage;

export const useWebsocket = () => {
  const [websocket, setWebsocket] = useState<WebSocket | null>(null);
  const peerId = useUIStore((state) => state.peerId);

  useEffect(() => {
    const path = `wss://${window.location.hostname}:${window.location.port}/ws/${peerId}`;
    console.log("ws path", path);
    const client = new WebSocket(path);
    client.onmessage = (message) => {
      console.debug("Websocket message", message.data);
      const decoded: SocketMessage = JSON.parse(message.data as string);
      switch (decoded.kind) {
        case "Pubs":
          useUIStore.setState((s) => ({
            ...s,
            pubs: decoded.list,
          }));
          break;
        // (rf/dispatch [:pubs (apply hash-map (flatten (map #(vector (:id %) %) (:list msg))))])
        // "Tables"
        // (rf/dispatch [:tables (apply hash-map (flatten (map #(vector (:id %) %) (:list msg))))])
        // "Pong" (do)
        // "Pub" (rf/dispatch [:pub (:data msg)])
        // "Table" (rf/dispatch [:table (:data msg)])
        // "Person" (rf/dispatch [:person (:data msg)])
        // "Data" (rf/dispatch [:msg (:author msg) (:content msg)]))))

        default:
          console.warn("unknown message", decoded);
      }
    };
    client.onopen = () => {
      console.debug("websocket connected");
      setWebsocket(client);
    };
    client.onerror = (error) => {
      console.debug("Websocket error", JSON.stringify(error));
    };
    client.onclose = (event) => {
      console.debug("Websocket closed", event);
    };
    return () => {
      if (websocket != null) {
        websocket.onopen = null;
        websocket.onclose = null;
        websocket.close();
      }
    };
  }, []);

  return websocket;
};
