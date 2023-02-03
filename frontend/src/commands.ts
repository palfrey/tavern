//   (defn ping [websocket]
//     (send-command websocket {"kind" "Ping"})
//     (= (.-readyState websocket) 1))

interface ListPubsCommand {
  kind: "ListPubs";
}

interface DeletePubCommand {
  kind: "DeletePub";
  pub_id: string;
}

interface JoinPubCommand {
  kind: "JoinPub";
  pub_id: string;
}

interface CreatePubCommand {
  kind: "CreatePub";
  name: string;
}

type Command =
  | ListPubsCommand
  | DeletePubCommand
  | JoinPubCommand
  | CreatePubCommand;

export const sendCommand = (websocket: WebSocket | null, msg: Command) => {
  const data = JSON.stringify(msg);
  if (websocket === null) {
    console.log("Attempt to send when null", data);
    return;
  }
  const state = websocket.readyState;
  if (state == 1) {
    console.log("Sending", data);
    websocket.send(data);
  } else {
    console.log("Attempt to send when not open", state, data);
    // create-ws
  }
};

export function listPubs(websocket: WebSocket | null): void {
  sendCommand(websocket, { kind: "ListPubs" });
}

export function createPub(websocket: WebSocket | null, name: string) {
  sendCommand(websocket, { kind: "CreatePub", name: name });
}
//   (defn create-pub [websocket name]
//     (send-command websocket {"kind" "CreatePub" "name" name}))

export function deletePub(websocket: WebSocket | null, pubId: string) {
  sendCommand(websocket, { kind: "DeletePub", pub_id: pubId });
}

export function joinPub(websocket: WebSocket | null, pubId: string) {
  sendCommand(websocket, { kind: "JoinPub", pub_id: pubId });
}

//   (defn join-pub [websocket pub_id]
//     (send-command websocket {"kind" "JoinPub" "pub_id" pub_id}))

//   (defn leave-pub [websocket]
//     (send-command websocket {"kind" "LeavePub"}))

//   (defn list-tables [websocket pub_id]
//     (send-command websocket {"kind" "ListTables" "pub_id" pub_id}))

//   (defn create-table [websocket pub_id name]
//     (send-command websocket {"kind" "CreateTable" "pub_id" pub_id "name" name}))

//   (defn join-table [websocket table_id]
//     (send-command websocket {"kind" "JoinTable" "table_id" table_id}))

//   (defn leave-table [websocket]
//     (send-command websocket {"kind" "LeaveTable"}))

//   (defn delete-table [websocket table_id]
//     (send-command websocket {"kind" "DeleteTable" "table_id" table_id}))

//   (defn get-person [websocket user_id]
//     (send-command websocket {"kind" "GetPerson" "user_id" user_id}))

//   (defn send [websocket user_id content]
//     (send-command websocket {"kind" "Send" "user_id" user_id "content" content}))
