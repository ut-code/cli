// src/main.tsx
import { program } from "commander";
import { render } from "ink";

// src/ui/components/Logo.tsx
import { Text } from "ink";
import { useEffect, useState } from "react";
import { jsxDEV } from "react/jsx-dev-runtime";
var Counter = () => {
  const [counter, setCounter] = useState(0);
  useEffect(() => {
    const timer = setInterval(() => {
      setCounter((previousCounter) => previousCounter + 1);
    }, 100);
    return () => {
      clearInterval(timer);
    };
  }, []);
  return /* @__PURE__ */ jsxDEV(Text, {
    color: "green",
    children: [
      counter,
      " tests passed"
    ]
  }, undefined, true, undefined, this);
};

// src/main.tsx
import { jsxDEV as jsxDEV2 } from "react/jsx-dev-runtime";
program.action(() => {
  render(/* @__PURE__ */ jsxDEV2(Counter, {}, undefined, false, undefined, this));
});
program.parse();
