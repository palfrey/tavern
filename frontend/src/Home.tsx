import { useState } from "react";
import { createPub, deletePub, joinPub, listPubs, WS } from "./commands";
import { useUIStore } from "./Store";

export default function Home({ websocket }: { websocket: WS }) {
  const [pubName, setPubName] = useState("");
  const pubs = useUIStore((s) => s.pubs);
  return (
    <div>
      <h1>Tavern</h1>
      <input
        type="button"
        className="btn btn-secondary"
        value="Update pub list"
        onClick={(evt) => {
          listPubs(websocket);
          evt.preventDefault();
        }}
      ></input>
      <div>Pubs</div>
      <ul>
        {pubs.map((pub) => (
          <li key={pub.id} className="pubItem">
            <span className="pubName">{pub.name}</span>
            <span>&nbsp;</span>
            <button
              id={"join-" + pub.name}
              className="btn btn-primary joinPub"
              onClick={(evt) => {
                joinPub(websocket, pub.id);
                evt.preventDefault();
              }}
            >
              Join
            </button>
            <span>&nbsp;</span>
            {pub.persons.length == 0 && (
              <button
                className="btn btn-danger deletePub"
                onClick={(evt) => {
                  deletePub(websocket, pub.id);
                  evt.preventDefault();
                }}
              >
                Delete
              </button>
            )}
          </li>
        ))}
      </ul>
      <form>
        <div className="form-group">
          <label htmlFor="pubName">New pub</label>
          <input
            type="text"
            className="form-control"
            id="pubName"
            placeholder="Enter pub name"
            value={pubName}
            onChange={(evt) => setPubName(evt.target.value)}
          />
        </div>
        <button
          id="createPub"
          type="button"
          className="btn btn-primary"
          onClick={(evt) => {
            createPub(websocket, pubName);
            evt.preventDefault();
          }}
        >
          Create pub
        </button>
      </form>
    </div>
  );
}
