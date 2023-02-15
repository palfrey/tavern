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
          console.debug("got stream", stream);
          setMs(stream);
        })
        .catch((err) => {
          console.warn("usermedia error", err);
          setLastError(err);
        });
    }
  }, [ms]);
  return { mediaStream: ms, lastError };
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
      console.warn(`Element is null for ${name}!`);
      return;
    }
    if (!(element instanceof HTMLVideoElement)) {
      console.warn(`Element is not a video for ${name}!`, element);
      return;
    }

    console.debug("update video", name, type, rtcpeer);
    if (type === "local") {
      element.srcObject = stream;
    } else {
      if (!(stream instanceof MediaStream)) {
        console.warn("wrong remote stream", type, stream);
        throw Error;
      }
      if (rtcpeer === null) {
        const config = {
          iceServers: [{ urls: "stun:stun.l.google.com:19302" }],
        };
        const conn = new RTCPeerConnection(config);
        const tracks = stream.getTracks();
        for (const track of tracks) {
          conn.addTrack(track, stream);
          conn.onicecandidate = (candidate) => {
            console.warn("candidate", candidate);
            if (candidate.candidate !== null) {
              send(websocket, name, JSON.stringify(candidate.candidate));
            }
          };
          conn.onnegotiationneeded = () => {
            conn.createOffer().then((offer) => {
              console.warn("offer", offer);
              conn.setLocalDescription(offer).then(() => {
                console.warn("local desc", conn.localDescription);
                send(websocket, name, JSON.stringify(conn.localDescription));
              });
            });
          };
          conn.ontrack = (event) => {
            const remoteStream = event.streams[0];
            console.warn("ontrack", event, remoteStream);
            element.srcObject = remoteStream;
          };
        }
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
  };
  useEffect(() => {
    update();
    return () => {
      console.debug("unmounting", name);
      if (rtcpeer != null) {
        rtcpeer.close();
        setRtcpeer(null);
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
  return <video id={name} autoPlay={true} />;
}

VideoComponent.displayName = "video-component";

function getStreams() {
  const peerId = useUIStore((s) => s.peerId);
  const mediaStream = useUIStore((s) => s.mediaStream);
  const peers = useUIStore((s) =>
    (s.currentTable()?.persons ?? []).filter((p) => p != peerId)
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
        key={peer}
        name={peer}
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
      <tbody style={{ width: "100%" }}>
        {[...Array(size).keys()].map((x) => (
          <tr key={`row-${x}`} style={{ width: "100%" }}>
            {[...Array(size).keys()].map((y) => {
              const idx = x * size + y;
              if (idx >= total) {
                return <React.Fragment key={`stream-${idx}`}></React.Fragment>;
              }
              const entry = streams[idx];
              return (
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
                  {entry}
                </td>
              );
            })}
          </tr>
        ))}
      </tbody>
    </table>
  );
}
