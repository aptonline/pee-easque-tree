import React from "react";
import { Ps3WaveBackground } from "./components/Ps3WaveBackground";

function App() {
  return (
    <Ps3WaveBackground
      gradient={["#00111f", "#002b45", "#004b75", "#007fc4"]}
      waveColor="#f5faff"
    >
      <div>
        <div style={{ textAlign: "center" }}>PS3 Wave React Demo</div>
        <div style={{ fontSize: "0.9em", opacity: 0.8 }}>
          Controlled via React props (gradient + waveColor)
        </div>
      </div>
    </Ps3WaveBackground>
  );
}

export default App;
