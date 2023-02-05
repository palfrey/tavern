import create from "zustand";
import { devtools, persist } from "zustand/middleware";
import { v4 as uuidv4 } from "uuid";
import { Pub, Table } from "./Data";

interface IUIStore {
  peerId: string;
  peers: { [key: string]: RTCPeerConnection };
  pubs: Pub[];
  currentPubId: string | null;
  tables: Table[];
  currentTableId: string | null;
  currentPub: () => Pub | null;
  currentTable: () => Table | null;
  mediaStream: MediaProvider | null;
}

export const useUIStore = create<IUIStore>()(
  persist(
    devtools(
      (set, get) =>
        ({
          peerId: uuidv4(),
          peers: {},
          pubs: [],
          tables: [],
          currentPubId: null,
          currentTableId: null,
          currentPub: () => {
            const id = get().currentPubId;
            if (id === null) {
              return null;
            }
            const matching = get().pubs.filter((p) => p.id == id);
            if (matching.length == 0) {
              return null;
            }
            return matching[0];
          },
          currentTable: () => null,
          mediaStream: null,
        } as IUIStore)
    )
  )
);
