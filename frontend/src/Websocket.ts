import { useUIStore } from "./Store";
import { listPubs, listTables } from "./commands";
import { doMessage, SocketMessage } from "./messages";
import { default as reactWebSocket } from "react-use-websocket";

export const useWebsocket = () => {
  const peerId = useUIStore((state) => state.peerId);
  const path = `wss://${window.location.hostname}:${window.location.port}/ws/${peerId}`;
  const peers = useUIStore((s) => s.peers);
  const currentPubId = useUIStore((s) => {
    const me = s.me();
    return me && me.pub_id;
  });
  const websocket = reactWebSocket(path, {
    share: true,
    onOpen: () => {
      console.debug("Websocket connected");
      listPubs(websocket);
      if (currentPubId !== null) {
        listTables(websocket, currentPubId);
      }
    },
    onMessage: (message: MessageEvent) => {
      console.warn("Websocket message", message.data);
      const decoded: SocketMessage = JSON.parse(message.data as string);
      let conn = null;
      if ("author" in decoded && decoded.author in peers) {
        conn = peers[decoded.author];
      }
      doMessage(websocket, conn, decoded);
    },
    filter: () => false,
  });
  return websocket;
};
