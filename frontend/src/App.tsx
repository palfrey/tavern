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
  console.debug("app store", JSON.stringify(store));
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
