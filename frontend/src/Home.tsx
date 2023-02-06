import { useState } from "react";
import { createPub, deletePub, joinPub, listPubs } from "./commands";
import { useUIStore } from "./Store";
import { useWebsocket } from "./Websocket";

export default function Home() {
  const [pubName, setPubName] = useState("");
  const pubs = useUIStore((s) => s.pubs);
  const websocket = useWebsocket();
  return (
    <div>
      <h1>Tavern</h1>
      <input
        type="button"
        className="btn btn-secondary"
        value="Update pub list"
        onClick={() => listPubs(websocket)}
      ></input>
      <div>Pubs</div>
      <ul>
        {pubs.map((pub) => (
          <li key={pub.id}>
            {pub.name}
            <span>&nbsp;</span>
            <button
              className="btn btn-primary"
              onClick={() => joinPub(websocket, pub.id)}
            >
              Join
            </button>
            <span>&nbsp;</span>
            {pub.persons.length == 0 && (
              <button
                className="btn btn-danger"
                onClick={() => deletePub(websocket, pub.id)}
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
          type="button"
          className="btn btn-primary"
          onClick={() => createPub(websocket, pubName)}
        >
          Create pub
        </button>
      </form>
    </div>
  );
}
