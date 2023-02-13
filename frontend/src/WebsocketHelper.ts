let websocket: WebSocket | null = null;
let current_url: string | null = null;
let openfunc: ((websocket: WebSocket) => void) | null = null;
let messagefunc: ((message: MessageEvent) => void) | null = null;
let messageQueue: string[] = [];

export class websocketWrapper {
  _configureOpen() {
    if (openfunc !== null && websocket !== null) {
      const funccopy = openfunc;
      websocket.onopen = () => {
        if (messageQueue.length !== 0) {
          console.info(`Buffer: ${messageQueue.length}`);
        }
        funccopy(websocket!);
      };
    }
  }

  _configureMessage() {
    if (messagefunc !== null && websocket !== null) {
      websocket.onmessage = messagefunc;
    }
  }

  connect(url: string | null) {
    if (url !== null) {
      if (websocket !== null && current_url !== url) {
        console.info(`URL change from ${current_url} to ${url}, swapping`);
        websocket.close();
        websocket = null;
      }
      current_url = url;
    } else {
      if (current_url === null) {
        console.warn("Can't connect as current_url is null");
        return;
      }
    }
    if (websocket === null || websocket.readyState === WebSocket.CLOSED) {
      console.info(`Connecting: ${current_url}`);
      websocket = new WebSocket(current_url);
      websocket.onerror = (error) => {
        console.debug("Websocket error", JSON.stringify(error));
      };
      websocket.onclose = (event) => {
        console.debug("Websocket closed", event);
      };
      this._configureOpen();
      this._configureMessage();
    }
  }

  setOpenFunc(func: (websocket: WebSocket) => void) {
    openfunc = func;
    this._configureOpen();
  }

  setMessageFunc(func: (message: MessageEvent) => void) {
    messagefunc = func;
    this._configureMessage();
  }

  send(message: string) {
    if (websocket != null && websocket.readyState === websocket.OPEN) {
      websocket.send(message);
    } else {
      console.info(`Buffering (${messageQueue.push(message)}): ${message}`);
      this.connect(null);
    }
  }
}

export const WebsocketWrapper = new websocketWrapper();
