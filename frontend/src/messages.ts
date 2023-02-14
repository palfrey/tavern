import { Person, Pub, Table } from "./Data";
import produce from "immer";
import { useUIStore } from "./Store";
import { send, WS } from "./commands";

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

interface CreateTableMessage {
  kind: "CreateTable";
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

export type SocketMessage =
  | PubsMessage
  | TablesMessage
  | PongMessage
  | CreatePubMessage
  | CreateTableMessage
  | PersonMessage
  | DataMessage;

function handleDataMsg(websocket: WS, peer: string, msg: any) {
  console.log("video msg from", peer, JSON.stringify(msg));
  let conn = useUIStore((s) => {
    if (peer in s.peers) {
      return s.peers[peer];
    } else {
      return null;
    }
  });
  if (conn === null) {
    console.warn(`Don't have a connection for ${peer}`);
    return;
  }
  if (msg === null) {
    console.log("Null video message from", peer);
  } else if (msg.type == "offer") {
    conn.setRemoteDescription(msg);
    conn.createAnswer().then((answer) => {
      console.log("answer", answer);
      conn!.setLocalDescription(answer).then(() => {
        send(websocket, peer, JSON.stringify(conn!.localDescription));
      });
    });
  } else if (msg.type == "answer") {
    conn.setRemoteDescription(msg);
  } else if (msg.candidate != null) {
    conn.addIceCandidate(msg);
  } else {
    console.log("video msg from", peer, JSON.stringify(msg));
  }
}

export const doMessage = (websocket: WS, message: SocketMessage) => {
  switch (message.kind) {
    case "Pubs":
      message.list.sort((a, b) => a.name.localeCompare(b.name));
      useUIStore.setState((s) => ({
        ...s,
        pubs: message.list,
      }));
      break;
    case "Tables":
      useUIStore.setState((s) => ({
        ...s,
        tables: message.list,
      }));
      break;
    case "Pong": {
      break;
    }
    case "CreatePub":
      useUIStore.setState((s) => ({
        ...s,
        pubs: [...s.pubs, message.data],
      }));
      break;
    case "CreateTable":
      useUIStore.setState((s) => ({
        ...s,
        tables: [...s.tables, message.data],
      }));
      break;
    case "Person":
      const person = message.data;
      useUIStore.setState(
        produce((s) => {
          s.persons[person.id] = person;
        })
      );
      break;
    case "Data": {
      handleDataMsg(websocket, message.author, message.content);
      break;
    }

    default:
      console.warn("unknown message", message);
  }
};
