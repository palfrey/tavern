import { doMessage, SocketMessage } from "./messages";
import { useUIStore } from "./Store";
import { default as reactUseWebsocket } from "react-use-websocket";
import { listPubs, listTables } from "./commands";

export const useWebsocket = () => {
  const peerId = useUIStore((state) => state.peerId);
  const path = `wss://${window.location.hostname}:${window.location.port}/ws/${peerId}`;
  const currentPubId = useUIStore((s) => {
    const me = s.me();
    return me && me.pub_id;
  });
  const websocket = reactUseWebsocket(path, {
    share: true,
    onOpen: () => {
      console.debug("websocket connected");
      listPubs(websocket);
      if (currentPubId !== null) {
        listTables(websocket, currentPubId);
      }
    },
    filter: () => false, // To fix re-render with last message https://github.com/robtaussig/react-use-websocket/issues/93#issuecomment-876702088
    onMessage: (message) => {
      console.debug("Websocket message", message.data);
      const decoded: SocketMessage = JSON.parse(message.data as string);
      doMessage(websocket, decoded);
    },
    onError: (error) => {
      console.debug("Websocket error", JSON.stringify(error));
    },
    onClose: (event) => {
      console.debug("Websocket closed", event);
    },
  });

  return websocket;
};
