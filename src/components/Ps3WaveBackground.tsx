import React, { useEffect, useRef, type ReactNode } from "react";
import * as THREE from "three";
import "./Ps3WaveBackground.css";

const waveClothVertexShader = `
float hash(float n){return fract(sin(n)*1e4);}
float hash(vec2 p){return fract(1e4*sin(17.*p.x+p.y*.1)*(.1+abs(sin(p.y*13.+p.x))));}

float noise(float x){
    float i=floor(x);
    float f=fract(x);
    float u=f*f*(3.-2.*f);
    return mix(hash(i),hash(i+1.),u);
}

float noise(vec2 x){
    vec2 i=floor(x);
    vec2 f=fract(x);
    float a=hash(i);
    float b=hash(i+vec2(1.,0.));
    float c=hash(i+vec2(0.,1.));
    float d=hash(i+vec2(1.,1.));
    vec2 u=f*f*(3.-2.*f);
    return mix(a,b,u.x)+(c-a)*u.y*(1.-u.x)+(d-b)*u.x*u.y;
}

float noise(vec3 x){
    const vec3 step=vec3(110,241,171);
    vec3 i=floor(x);
    vec3 f=fract(x);
    float n=dot(i,step);
    vec3 u=f*f*(3.-2.*f);
    return mix(mix(mix(hash(n+dot(step,vec3(0,0,0))),hash(n+dot(step,vec3(1,0,0))),u.x),
    mix(hash(n+dot(step,vec3(0,1,0))),hash(n+dot(step,vec3(1,1,0))),u.x),u.y),
    mix(mix(hash(n+dot(step,vec3(0,0,1))),hash(n+dot(step,vec3(1,0,1))),u.x),
    mix(hash(n+dot(step,vec3(0,1,1))),hash(n+dot(step,vec3(1,1,1))),u.x),u.y),u.z);
}

uniform float uTime;
varying vec3 vPosition;

float xmbNoise(vec3 x){
    return cos(x.z*4.)*cos(x.z+uTime/10.+x.x);
}

void main(){
    vec3 p=vec3(position.x,0.,position.y);

    // base noise wave
    p.y = xmbNoise(p)/8.;

    // distort
    vec3 p2=p;
    p2.x-=uTime/5.;
    p2.x/=4.;
    p2.y-=uTime/100.;
    p2.z-=uTime/10.;
    p.y-=noise(p2*8.)/12.+cos(p.x*2.-uTime/2.)/5.-.3;
    p.z-=noise(p2*8.)/12.;

    vec4 modelPosition=modelMatrix*vec4(p,1.);
    vec4 viewPosition=viewMatrix*modelPosition;
    vec4 projectedPosition=projectionMatrix*viewPosition;
    gl_Position=projectedPosition;

    vPosition=p;
}
`;

const waveClothFragmentShader = `
uniform vec3 uColor;
varying vec3 vPosition;

vec3 computeNormal(vec3 normal){
    vec3 X = dFdx(normal);
    vec3 Y = dFdy(normal);
    return normalize(cross(X, Y));
}

float fresnel(float bias,float scale,float power,vec3 I,vec3 N)
{
    return bias+scale*pow(1.+dot(I,N),power);
}

void main(){
    vec3 color = uColor;

    vec3 cNormal = computeNormal(vPosition);
    vec3 eyeVector = vec3(0.,0.,-1.);
    float F = fresnel(0.,.5,4.,eyeVector,cNormal);
    float alpha = F * 0.6;

    gl_FragColor = vec4(color, alpha);
}
`;

export type Ps3WaveBackgroundProps = {
  waveColor?: string;
  className?: string;
  children?: ReactNode;
};

export const Ps3WaveBackground: React.FC<Ps3WaveBackgroundProps> = ({
  waveColor = "#ffffff",
  className = "",
  children,
}) => {
  const waveContainerRef = useRef<HTMLDivElement | null>(null);
  const uniformsRef = useRef<{ uTime: { value: number }; uColor: THREE.Color | any } | null>(null);

  useEffect(() => {
    if (uniformsRef.current) {
      // uColor is stored as a THREE.Color wrapped in { value }
      (uniformsRef.current as any).uColor.value.set(waveColor);
    }
  }, [waveColor]);

  useEffect(() => {
    const containerEl = waveContainerRef.current;
    if (!containerEl) return;

    const scene = new THREE.Scene();
    // Zoom in by reducing camera bounds (more zoomed in to show wave detail like PS3)
    // Use smaller vertical bounds to ensure wave fills the header height
    const camera = new THREE.OrthographicCamera(-0.6, 0.6, 0.3, -0.3, 0.1, 10);
    camera.position.set(0, 0, 2);
    camera.lookAt(0, 0, 0);

    const renderer = new THREE.WebGLRenderer({
      alpha: true,
      antialias: true,
    });
    renderer.setClearColor(0x000000, 0);
    containerEl.innerHTML = "";
    containerEl.appendChild(renderer.domElement);

    const uniforms = {
      uTime: { value: 0 },
      uColor: { value: new THREE.Color(waveColor) },
    };
    uniformsRef.current = uniforms as any;

    const material = new THREE.ShaderMaterial({
      vertexShader: waveClothVertexShader,
      fragmentShader: waveClothFragmentShader,
      uniforms,
      side: THREE.DoubleSide,
      transparent: true,
      depthTest: false,
    });

    // Enable derivatives extension for normal computation in fragment shader
    (material as any).extensions = { derivatives: true };

    const geometry = new THREE.PlaneGeometry(2, 2, 128, 128);
    const plane = new THREE.Mesh(geometry, material);
    scene.add(plane);

    const resize = () => {
      const { clientWidth, clientHeight } = containerEl;
      if (!clientWidth || !clientHeight) return;
      const dpr = window.devicePixelRatio || 1;
      renderer.setSize(clientWidth, clientHeight, false);
      renderer.setPixelRatio(dpr);
    };
    resize();

    const clock = new THREE.Clock();
    let stopped = false;

    const animate = () => {
      if (stopped) return;
      uniforms.uTime.value = clock.getElapsedTime();
      renderer.render(scene, camera);
      requestAnimationFrame(animate);
    };
    animate();

    const handleResize = () => resize();
    window.addEventListener("resize", handleResize);

    return () => {
      stopped = true;
      window.removeEventListener("resize", handleResize);
      geometry.dispose();
      material.dispose();
      renderer.dispose();
      if (renderer.domElement.parentNode === containerEl) {
        containerEl.removeChild(renderer.domElement);
      }
      uniformsRef.current = null;
    };
  }, []);

  return (
    <div className={`ps3-wave-container ${className}`}>
      <div className="ps3-wave-canvas" ref={waveContainerRef} />
      {children}
    </div>
  );
};
