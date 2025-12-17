# PS3 Wave React Demo (Vite + local three.js)

This is a small React + Vite project that wraps the PS3-style startup wave
background into a reusable React component, suitable for use in a Tauri app.

- Gradient colors are controlled via **props → CSS variables**
- Wave color is controlled via a **Three.js uniform**, also via props

## Quick start

```bash
# inside this folder
npm install
npm run dev
```

Then open the URL Vite prints (usually http://localhost:5173).

You should see:

- An animated multi-color gradient background
- A noisy, translucent wave band (PS3-style) moving across it
- Text over the top rendered as normal React children

## Component API

`src/components/Ps3WaveBackground.tsx` exports:

```ts
type Ps3WaveBackgroundProps = {
  gradient?: [string, string, string, string]; // 4 CSS colors
  waveColor?: string;                          // CSS color / hex
  children?: React.ReactNode;                  // overlay UI
};
```

Usage (see `src/App.tsx`):

```tsx
<Ps3WaveBackground
  gradient={["#00111f", "#002b45", "#004b75", "#007fc4"]}
  waveColor="#f5faff"
>
  <div>My app UI here</div>
</Ps3WaveBackground>
```

## Using in a Tauri + React app

1. Copy the following into your Tauri React project:

   - `src/components/Ps3WaveBackground.tsx`
   - `src/ps3-bg.css`

2. Make sure you have `three` installed:

   ```bash
   npm install three
   ```

3. Import the CSS once (e.g. in your root or the component file already does it):

   ```ts
   import "../ps3-bg.css";
   ```

4. Use the component anywhere in your React tree:

   ```tsx
   import { Ps3WaveBackground } from "./components/Ps3WaveBackground";

   function App() {
     return (
       <Ps3WaveBackground
         gradient={["#120c2c", "#3a0ca3", "#7209b7", "#f72585"]}
         waveColor="#ffffff"
       >
         {/* Tauri app UI */}
       </Ps3WaveBackground>
     );
   }
   ```

Because everything is just React + Three, it works the same in a browser or
inside Tauri's WebView.
