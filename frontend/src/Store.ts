import create from "zustand";
import { devtools, persist } from "zustand/middleware";
import { v4 as uuidv4 } from "uuid";
import { Pub } from "./Data";

interface IUIStore {
  peerId: string;
  pubs: Pub[];
}

export const useUIStore = create<IUIStore>()(
  persist(
    devtools(
      () =>
        ({
          peerId: uuidv4(),
          pubs: [],
        } as IUIStore)
    )
  )
);
