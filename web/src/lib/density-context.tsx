import { createContext, type ParentComponent, useContext } from "solid-js";
import type { DensityMode } from "./design-tokens";
import { prefStore } from "./store";

/**
 * Density Context Provider
 *
 * Provides density mode to all child components.
 * Reads from user preferences and applies the appropriate density class to the container.
 *
 * Components can override density locally via props.
 */
const DensityContext = createContext<DensityMode>("comfortable");

export const DensityProvider: ParentComponent = (props) => {
  const density = () => (prefStore.densityMode?.() as DensityMode) || "comfortable";

  return (
    <DensityContext.Provider value={density()}>
      <div class={`density-${density()}`}>{props.children}</div>
    </DensityContext.Provider>
  );
};

export const useDensity = () => useContext(DensityContext);
