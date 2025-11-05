import React, { useRef, useMemo, useState, useEffect } from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { fragmentShader, vertexShader } from "../shaders/backgroundShader";
import "./ShaderBackground.css";

interface ShaderBackgroundProps {
    speed?: number;
    intensity?: number;
    scale?: number;
    opacity?: number;
    enabled?: boolean;
}

function ShaderPlane({
    speed = 1.0,
    intensity = 1.0,
    scale = 1.0,
    opacity = 1.0,
    isPaused,
}: Omit<ShaderBackgroundProps, "enabled"> & { isPaused: boolean }) {
    const meshRef = useRef<THREE.Mesh>(null);
    const startTime = useRef(Date.now());
    const lastFrameTime = useRef(Date.now());
    const targetFPS = 30; // Throttle to 30 FPS for performance
    const frameInterval = 1000 / targetFPS;

    const uniforms = useMemo(
        () => ({
            iTime: { value: 0 },
            iResolution: {
                value: new THREE.Vector2(window.innerWidth, window.innerHeight),
            },
            uSpeed: { value: speed },
            uIntensity: { value: intensity },
            uScale: { value: scale },
            uOpacity: { value: opacity },
        }),
        [],
    );

    // Update uniforms when props change
    React.useEffect(() => {
        if (uniforms.uSpeed) uniforms.uSpeed.value = speed;
        if (uniforms.uIntensity) uniforms.uIntensity.value = intensity;
        if (uniforms.uScale) uniforms.uScale.value = scale;
        if (uniforms.uOpacity) uniforms.uOpacity.value = opacity;
    }, [speed, intensity, scale, opacity, uniforms]);

    // Update resolution on window resize
    React.useEffect(() => {
        const handleResize = () => {
            if (uniforms.iResolution) {
                uniforms.iResolution.value.set(
                    window.innerWidth,
                    window.innerHeight,
                );
            }
        };

        window.addEventListener("resize", handleResize);
        return () => window.removeEventListener("resize", handleResize);
    }, [uniforms]);

    // Animation loop with FPS throttling and pause support
    useFrame(() => {
        if (!meshRef.current || isPaused) return;

        const now = Date.now();
        const delta = now - lastFrameTime.current;

        // Throttle to target FPS
        if (delta >= frameInterval) {
            lastFrameTime.current = now - (delta % frameInterval);
            const elapsed = (now - startTime.current) / 1000;
            if (uniforms.iTime) {
                uniforms.iTime.value = elapsed;
            }
        }
    });

    return (
        <mesh ref={meshRef} scale={[2, 2, 1]}>
            <planeGeometry args={[2, 2]} />
            <shaderMaterial
                vertexShader={vertexShader}
                fragmentShader={fragmentShader}
                uniforms={uniforms}
                transparent={true}
                depthWrite={false}
            />
        </mesh>
    );
}

export function ShaderBackground({
    speed = 0.3,
    intensity = 1.0,
    scale = 1.5,
    opacity = 0.4,
    enabled = true,
}: ShaderBackgroundProps) {
    const [isPaused, setIsPaused] = useState(false);

    // Pause animation when window is not visible
    useEffect(() => {
        const handleVisibilityChange = () => {
            setIsPaused(document.hidden);
        };

        document.addEventListener("visibilitychange", handleVisibilityChange);
        return () => {
            document.removeEventListener("visibilitychange", handleVisibilityChange);
        };
    }, []);

    // Detect low-end devices and reduce quality
    const isLowEndDevice = useMemo(() => {
        // Check for low memory or reduced motion preference
        const hasLowMemory = (navigator as any).deviceMemory && (navigator as any).deviceMemory < 4;
        const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
        return hasLowMemory || prefersReducedMotion;
    }, []);

    if (!enabled) return null;

    // Adjust DPR for performance on low-end devices
    const dpr = isLowEndDevice ? 1 : Math.min(window.devicePixelRatio, 2);

    return (
        <div className="shader-background">
            <Canvas
                camera={{ position: [0, 0, 1], fov: 75 }}
                gl={{
                    alpha: true,
                    antialias: !isLowEndDevice, // Disable AA on low-end devices
                    powerPreference: isLowEndDevice ? "low-power" : "high-performance",
                }}
                dpr={dpr}
                frameloop={isPaused ? "never" : "always"}
            >
                <ShaderPlane
                    speed={speed}
                    intensity={intensity}
                    scale={scale}
                    opacity={opacity}
                    isPaused={isPaused}
                />
            </Canvas>
        </div>
    );
}

export default ShaderBackground;
