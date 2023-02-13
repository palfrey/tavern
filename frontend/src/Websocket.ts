import { useUIStore } from "./Store";
import { listPubs, listTables } from "./commands";
import { WebsocketWrapper } from "./WebsocketHelper";
import { doMessage, SocketMessage } from "./messages";

export const useWebsocket = () => {
  const peerId = useUIStore((state) => state.peerId);
  const path = `wss://${window.location.hostname}:${window.location.port}/ws/${peerId}`;
  console.log(`ws path: ${path}`);
  const currentPubId = useUIStore((s) => {
    const me = s.me();
    return me && me.pub_id;
  });
  WebsocketWrapper.setOpenFunc((websocket: WebSocket) => {
    console.debug("Websocket connected");
    listPubs(WebsocketWrapper);
    if (currentPubId !== null) {
      listTables(WebsocketWrapper, currentPubId);
    }
  });
  WebsocketWrapper.setMessageFunc((message: MessageEvent) => {
    console.debug("Websocket message", message.data);
    const decoded: SocketMessage = JSON.parse(message.data as string);
    doMessage(WebsocketWrapper, decoded);
  });
  WebsocketWrapper.connect(path);
  return WebsocketWrapper;
};
