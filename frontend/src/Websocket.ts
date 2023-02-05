import produce from "immer";
import { useEffect, useState } from "react";
import { Person, Pub, Table } from "./Data";
import { useUIStore } from "./Store";

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

interface CreatePubMessage {
  kind: "CreatePub";
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
  | CreatePubMessage
  | TableMessage
  | PersonMessage
  | DataMessage;

export const useWebsocket = () => {
  const [websocket, setWebsocket] = useState<WebSocket | null>(null);
  const peerId = useUIStore((state) => state.peerId);

  useEffect(() => {
    if (websocket === null) {
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
          // (rf/dispatch <pubs (apply hash-map (flatten (map {() => vector (id=%) %) (list=msg))))])
          // "Tables"
          // (rf/dispatch <tables (apply hash-map (flatten (map {() => vector (id=%) %) (list=msg))))])
          // "Pong" (do)
          case "CreatePub":
            useUIStore.setState((s) => ({
              ...s,
              pubs: [...s.pubs, decoded.data],
            }));
            break;
          // "Table" (rf/dispatch <table (data=msg)])
          case "Person":
            const person = decoded.data;
            useUIStore.setState(
              produce((s) => {
                s.persons[person.id] = person;
              })
            );
            break;
          // "Data" (rf/dispatch :msg (author=msg) (content=msg)]))))

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
        setWebsocket(null);
      };
      return () => {
        if (websocket != null) {
          console.log("Closing websocket");
          websocket.onopen = null;
          websocket.onclose = null;
          websocket.close();
          setWebsocket(null);
        }
      };
    }
  }, [websocket]);

  return websocket;
};
