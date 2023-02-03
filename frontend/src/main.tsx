import { render } from "react-dom";
import App from "./App";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const root = document.getElementById("root");
const queryClient = new QueryClient();
render(
  <QueryClientProvider client={queryClient}>
    <App />
  </QueryClientProvider>,
  root
);
