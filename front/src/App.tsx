import { useEffect } from "react";
import { Outlet, useNavigate } from "react-router-dom";
import { useBearStore } from "@/stores";
import { CommandPanel } from "./command";

import "./styles/index.css";

function App() {
  const store = useBearStore((state) => ({
    getUserConfig: state.getUserConfig,
  }));

  const navigate = useNavigate();

  useEffect(() => {
    store.getUserConfig().then((cfg: UserConfig) => {
      const { theme, customize_style } = cfg;

      if (theme === "system") {
        document.documentElement.dataset.colorScheme = window.matchMedia("(prefers-color-scheme: dark)").matches
          ? "dark"
          : "light";
      } else {
        document.documentElement.dataset.colorScheme = theme;
      }

      customize_style &&
        Object.keys(customize_style).length &&
        Object.keys(customize_style).forEach((key: string) => {
          document.documentElement.style.setProperty(
            `--reading-editable-${key.replace(/_/gi, "-")}`,
            customize_style[key as keyof CustomizeStyle] as string
          );
        });
    });
  }, []);

  return (
    <>
      <div className="h-full max-h-full ">
        <Outlet />
      </div>
      <CommandPanel />
    </>
  );
}

export default App;
