import { useEffect, useState } from "react";
import { Outlet, useLocation, useNavigate } from "react-router-dom";
import { useUIStore } from "./Store";

function Clock() {
  const [date, setDate] = useState(new Date());

  function refreshClock() {
    setDate(new Date());
  }
  useEffect(() => {
    const timerId = setInterval(refreshClock, 1000);
    return function cleanup() {
      clearInterval(timerId);
    };
  }, []);

  return <div>{date.toTimeString().split(" ")[0]}</div>;
}

function Core() {
  const location = useLocation();
  const navigate = useNavigate();
  const me = useUIStore((s) => s.me());
  useEffect(() => {
    console.log("pathname", location.pathname);
    if (me === null || me.pub_id === null) {
      if (location.pathname !== "/Home") {
        navigate("/Home");
      }
    } else if (me !== null && me.pub_id !== null) {
      if (me.table_id !== null) {
        if (location.pathname !== "/Table") {
          console.log("/Table");
          navigate("/Table");
        }
      } else if (location.pathname !== "/Pub") {
        console.log("/Pub");
        navigate("/Pub");
      }
    }
  }, [me, location, navigate]);
  return (
    <div>
      <nav className="navbar navbar-expand-md navbar-dark bg-dark fixed-top">
        <a className="navbar-brand" href="#">
          Tavern
        </a>
        <button
          className="navbar-toggler"
          type="button"
          data-toggle="collapse"
          data-target="#navbarsExampleDefault"
          aria-controls="navbarsExampleDefault"
          aria-expanded="false"
          aria-label="Toggle navigation"
        >
          <span className="navbar-toggler-icon"></span>
        </button>
        <div className="collapse navbar-collapse" id="navbarsExampleDefault">
          <ul className="navbar-nav mr-auto">
            <li className="nav-item active">
              <a className="nav-link" href="#">
                Home
                <span className="sr-only">(current)</span>
              </a>
            </li>
            <li className="nav-item">
              <a className="nav-link" href="#">
                Link
              </a>
            </li>
          </ul>
          <span className="navbar-text">
            <Clock />
          </span>
        </div>
      </nav>
      <main role="main" className="container-fluid">
        <Outlet />
      </main>
    </div>
  );
}

export default Core;
