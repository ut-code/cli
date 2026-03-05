#!/usr/bin/env node
// src/main.tsx
import { program } from "commander";
import { render } from "ink";

// src/ui/components/Greeting.tsx
import { Box } from "ink";
import BigText from "ink-big-text";
import { jsxDEV, Fragment } from "react/jsx-dev-runtime";
var Greeting = () => {
  return /* @__PURE__ */ jsxDEV(Fragment, {
    children: [
      /* @__PURE__ */ jsxDEV(BigText, {
        text: "We're",
        colors: ["#ffffff", "#00ff87"]
      }, undefined, false, undefined, this),
      /* @__PURE__ */ jsxDEV(Box, {
        children: [
          /* @__PURE__ */ jsxDEV(BigText, {
            text: "ut."
          }, undefined, false, undefined, this),
          /* @__PURE__ */ jsxDEV(BigText, {
            text: "code",
            colors: ["green"]
          }, undefined, false, undefined, this),
          /* @__PURE__ */ jsxDEV(BigText, {
            text: "();"
          }, undefined, false, undefined, this)
        ]
      }, undefined, true, undefined, this)
    ]
  }, undefined, true, undefined, this);
};

// src/main.tsx
import { jsxDEV as jsxDEV2 } from "react/jsx-dev-runtime";
program.action(() => {
  render(/* @__PURE__ */ jsxDEV2(Greeting, {}, undefined, false, undefined, this));
});
program.parse();
