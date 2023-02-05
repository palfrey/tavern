import create from "zustand";
import { devtools, persist } from "zustand/middleware";
import { v4 as uuidv4 } from "uuid";
import { Person, Pub, Table } from "./Data";

interface IUIStore {
  peerId: string;
  peers: { [key: string]: RTCPeerConnection };
  pubs: Pub[];
  tables: Table[];
  me: () => Person | null;
  currentPub: () => Pub | null;
  currentTable: () => Table | null;
  mediaStream: MediaProvider | null;
  persons: { [key: string]: Person };
}

export const useUIStore = create<IUIStore>()(
  persist(
    devtools(
      (set, get) =>
        ({
          peerId: uuidv4(),
          peers: {},
          pubs: [],
          persons: {},
          tables: [],
          me: () => {
            {
              if (get().peerId in get().persons) {
                return get().persons[get().peerId];
              } else {
                return null;
              }
            }
          },
          currentPub: () => {
            const me = get().me();
            const id = me && me.pub_id;
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
