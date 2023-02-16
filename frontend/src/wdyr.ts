import * as React from "react";

const { default: wdyr } = await import("@welldone-software/why-did-you-render");

wdyr(React, {
  include: [/.*/],
  exclude: [/^BrowserRouter/, /^Link/, /^Route/, /^RenderedRoute/],
  trackHooks: true,
  trackAllPureComponents: true,
  logOnDifferentValues: true,
});
