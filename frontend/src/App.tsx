import { useUIStore } from "./Store";
import {
  RouterProvider,
  useRouteError,
  createHashRouter,
  redirect,
} from "react-router-dom";
import Core from "./Core";
import Home from "./Home";
import About from "./About";
import { Pub } from "./Pub";
import { useEffect } from "react";
import { useMediaStreamWrapper } from "./Video";
import { Table } from "./Table";
import { ping } from "./commands";
import { useWebsocket } from "./Websocket";

function ErrorPage() {
  const error = useRouteError() as Response;

  return (
    <div id="error-page" data-testid="error-page">
      <h1>Oops!</h1>
      <p>Sorry, an unexpected error has occurred.</p>
      <p>
        <i>{JSON.stringify(error)}</i>
      </p>
    </div>
  );
}

function App() {
  const store = useUIStore.getState();
  console.debug("app store", store);
  const mediaStream = useUIStore((s) => s.mediaStream);
  const newMediaStream = useMediaStreamWrapper();
  const websocket = useWebsocket();

  useEffect(() => {
    const interval = setInterval(() => {
      ping(websocket);
    }, 5000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    if (mediaStream === null && newMediaStream.mediaStream !== null) {
      useUIStore.setState((s) => ({
        ...s,
        mediaStream: newMediaStream.mediaStream,
      }));
    }
  }, [mediaStream, newMediaStream.mediaStream]);

  const router = createHashRouter([
    {
      path: "/",
      element: <Core />,
      // errorElement: <ErrorPage />,
      loader: async (request) => {
        if (request.request.url.endsWith("/")) {
          console.debug("redirect for home", request.request.url);
          return redirect("/Home");
        }
        return null;
      },
      children: [
        { path: "Home", element: <Home /> },
        { path: "Pub", element: <Pub /> },
        { path: "Table", element: <Table /> },
        {
          path: "about",
          element: <About />,
        },
      ],
    },
  ]);
  return <RouterProvider router={router} />;
}

export default App;
