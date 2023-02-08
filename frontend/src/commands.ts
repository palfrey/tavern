import { WebSocketHook } from "react-use-websocket/dist/lib/types";

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

interface LeavePubCommand {
  kind: "LeavePub";
  pub_id: string;
}

interface ListTablesCommand {
  kind: "ListTables";
  pub_id: string;
}
interface CreateTableCommand {
  kind: "CreateTable";
  pub_id: string;
  name: string;
}
interface JoinTableCommand {
  kind: "JoinTable";
  table_id: string;
}
interface LeaveTableCommand {
  kind: "LeaveTable";
  table_id: string;
}
interface DeleteTableCommand {
  kind: "DeleteTable";
  table_id: string;
}
interface GetPersonCommand {
  kind: "GetPerson";
  user_id: string;
}
interface SendCommand {
  kind: "Send";
  user_id: string;
  content: string;
}
interface PingCommand {
  kind: "Ping";
}

type Command =
  | ListPubsCommand
  | DeletePubCommand
  | JoinPubCommand
  | CreatePubCommand
  | LeavePubCommand
  | ListTablesCommand
  | CreateTableCommand
  | JoinTableCommand
  | LeaveTableCommand
  | DeleteTableCommand
  | GetPersonCommand
  | SendCommand
  | PingCommand;

export const sendCommand = (websocket: WebSocketHook, msg: Command) => {
  const data = JSON.stringify(msg);
  console.log("Sending", data);
  websocket.sendMessage(data);
};

export function listPubs(websocket: WebSocketHook): void {
  sendCommand(websocket, { kind: "ListPubs" });
}

export function createPub(websocket: WebSocketHook, name: string) {
  sendCommand(websocket, { kind: "CreatePub", name: name });
}

export function deletePub(websocket: WebSocketHook, pubId: string) {
  sendCommand(websocket, { kind: "DeletePub", pub_id: pubId });
}

export function joinPub(websocket: WebSocketHook, pubId: string) {
  sendCommand(websocket, { kind: "JoinPub", pub_id: pubId });
}

export function leavePub(websocket: WebSocketHook, pubId: string) {
  sendCommand(websocket, { kind: "LeavePub", pub_id: pubId });
}

export function listTables(websocket: WebSocketHook, pubId: string) {
  sendCommand(websocket, { kind: "ListTables", pub_id: pubId });
}

export function createTable(
  websocket: WebSocketHook,
  pubId: string,
  name: string
) {
  sendCommand(websocket, { kind: "CreateTable", pub_id: pubId, name });
}

export function joinTable(websocket: WebSocketHook, tableId: string) {
  sendCommand(websocket, { kind: "JoinTable", table_id: tableId });
}

export function leaveTable(websocket: WebSocketHook, tableId: string) {
  sendCommand(websocket, { kind: "LeaveTable", table_id: tableId });
}

export function deleteTable(websocket: WebSocketHook, tableId: string) {
  sendCommand(websocket, { kind: "DeleteTable", table_id: tableId });
}

export function getPerson(websocket: WebSocketHook, userId: string) {
  sendCommand(websocket, { kind: "GetPerson", user_id: userId });
}

export function send(
  websocket: WebSocketHook,
  userId: string,
  content: string
) {
  sendCommand(websocket, { kind: "Send", user_id: userId, content });
}

export function ping(websocket: WebSocketHook) {
  sendCommand(websocket, { kind: "Ping" });
}
