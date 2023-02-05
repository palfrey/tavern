import React, { useEffect, useState } from "react";
import { send } from "./commands";
import { useUIStore } from "./Store";
import { useWebsocket } from "./Websocket";

export function useMediaStreamWrapper() {
  const [ms, setMs] = useState<MediaStream | null>(null);
  const [lastError, setLastError] = useState<object | null>(null);
  useEffect(() => {
    if (ms === null) {
      navigator.mediaDevices
        .getUserMedia({ video: true, audio: false })
        .then((stream) => {
          console.log("got stream", stream);
          setMs(stream);
        })
        .catch((err) => {
          console.log("usermedia error", err);
          setLastError(err);
        });
    }
  }, [ms]);
  return { mediaStream: ms, lastError };
}

// (ti/reg-event-db
//   :msg
//   (fn [db [_ peer msg]]
//     (if-let [conn (get-in db [:peers peer :connection])]
//       (video/handle-msg peer conn (.parse js/JSON msg)))
//     db))

function handleMsg(
  websocket: WebSocket,
  peer: string,
  conn: RTCPeerConnection,
  msg: any
) {
  console.log("video msg from", peer, JSON.stringify(msg));
  if (msg === null) {
    console.log("Null video message from", peer);
  } else if (msg.type == "offer") {
    conn.setRemoteDescription(msg);
    conn.createAnswer().then((answer) => {
      console.log("answer", answer);
      conn.setLocalDescription(answer).then(() => {
        send(websocket, peer, JSON.stringify(conn.localDescription));
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

function VideoComponent({
  name,
  type,
  stream,
}: {
  name: string;
  type: "local" | "remote";
  stream: MediaProvider;
}) {
  const websocket = useWebsocket();
  const [rtcpeer, setRtcpeer] = useState<RTCPeerConnection | null>(null);
  const update = () => {
    const element = document.getElementById(name);
    if (element === null) {
      console.log("Element is null!");
      return;
    }
    if (!(element instanceof HTMLVideoElement)) {
      console.log("Element is not a video!", element);
      return;
    }
    // new-argv (reagent/argv comp)
    // {keys=[stream type localstream]} (last new-argv)]
    const type: string = "";

    console.log("update video", name);
    if (type === "local") {
      element.srcObject = stream;
    } else {
      if (!(stream instanceof MediaStream)) {
        console.log("wrong remote stream", type, stream);
        throw Error;
      }
      if (rtcpeer === null) {
        const config = {
          iceServers: [{ urls: "stun:stun.l.google.com:19302" }],
        };
        const conn = new RTCPeerConnection(config);
        const tracks = stream.getTracks();
        for (var track of tracks) {
          conn.addTrack(track, stream);
          conn.onicecandidate = (candidate) => {
            console.log("candidate", candidate);
            if (candidate.candidate !== null) {
              send(websocket, name, JSON.stringify(candidate.candidate));
            }
          };
          conn.onnegotiationneeded = () => {
            conn.createOffer().then((offer) => {
              console.log("offer", offer);
              conn.setLocalDescription(offer).then(() => {
                console.log("local desc", conn.localDescription);
                send(websocket, name, JSON.stringify(conn.localDescription));
              });
            });
          };
          conn.ontrack = (event) => {
            const remoteStream = event.streams[0];
            console.log("ontrack", event, remoteStream);
            element.srcObject = remoteStream;
          };
          setRtcpeer(conn);
          useUIStore.setState((s) => ({
            ...s,
            peers: {
              ...s.peers,
              name: conn,
            },
          }));
        }
      }
    }
  };
  useEffect(() => {
    update();
    return () => {
      console.log("unmounting", name);
      if (rtcpeer != null) {
        rtcpeer.close();
        setRtcpeer(null);
      }
    };
  });
  return <video id={name} autoPlay={true} />;
}

VideoComponent.displayName = "video-component";

function getStreams() {
  const peerId = useUIStore((s) => s.peerId);
  const mediaStream = useUIStore((s) => s.mediaStream);
  const peers = useUIStore((s) =>
    s.currentTable()!.persons.filter((p) => p.id != peerId)
  );
  if (mediaStream === null) {
    return [];
  }

  const ret = [
    <VideoComponent
      key={peerId}
      name={peerId}
      type="local"
      stream={mediaStream}
    />,
  ];
  ret.push(
    ...peers.map((peer) => (
      <VideoComponent
        key={peer.id}
        name={peer.id}
        type="remote"
        stream={mediaStream}
      />
    ))
  );
  return ret;
}

export function Videos() {
  const streams = getStreams();
  const total = streams.length;
  const size = Math.ceil(total > 0 ? Math.sqrt(total) : 0);
  console.log("size", size);
  console.log("streams", streams);
  return (
    <table
      width="100%"
      style={{
        backgroundColor: "black",
        border: 1,
        borderStyle: "solid",
        borderColor: "black",
      }}
    >
      <tbody width="100%">
        {[...Array(size)].map((x) => (
          <tr key={`row-${x}`} width="100%">
            {[...Array(size)].map((y) => {
              const idx = x * size + y;
              if (idx >= total) {
                return <React.Fragment></React.Fragment>;
              }
              const entry = streams[idx];
              <td
                key={`stream-${idx}`}
                style={{
                  border: 1,
                  borderStyle: "solid",
                  borderColor: "black",
                }}
                width={size / 100}
              >
                <div style={{ color: "white" }}>stream-{idx}</div>
                entry
              </td>;
            })}
          </tr>
        ))}
      </tbody>
    </table>
  );
}
