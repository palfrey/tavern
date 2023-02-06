import { useEffect, useState } from "react";
import { singletonHook } from "react-singleton-hook";
import { doMessage, SocketMessage } from "./messages";
import { useUIStore } from "./Store";

const useWebsocketImpl = () => {
  const [websocket, setWebsocket] = useState<WebSocket | null>(null);
  const [settingUp, setSettingUp] = useState(false);
  const peerId = useUIStore((state) => state.peerId);

  useEffect(() => {
    if (websocket === null && !settingUp) {
      const path = `wss://${window.location.hostname}:${window.location.port}/ws/${peerId}`;
      console.log("ws path", path);
      const client = new WebSocket(path);
      client.onmessage = (message) => {
        console.debug("Websocket message", message.data);
        const decoded: SocketMessage = JSON.parse(message.data as string);
        doMessage(client, decoded);
      };
      client.onopen = () => {
        console.debug("websocket connected");
        setWebsocket(client);
      };
      client.onerror = (error) => {
        console.debug("Websocket error", JSON.stringify(error));
        if (websocket === null) {
          setSettingUp(false);
        }
      };
      client.onclose = (event) => {
        console.debug("Websocket closed", event);
        setWebsocket(null);
        setSettingUp(false);
      };
      setSettingUp(true);
      return () => {
        if (websocket != null) {
          console.log("Closing websocket");
          client.onopen = null;
          client.onclose = null;
          client.close();
          setWebsocket(null);
        }
      };
    }
  }, [websocket, settingUp]);

  return websocket;
};

export const useWebsocket = singletonHook(null, useWebsocketImpl);
