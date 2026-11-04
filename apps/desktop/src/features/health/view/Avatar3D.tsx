import { useEffect, useRef, useState, type ReactNode } from "react";
import type { AvatarSpec } from "../viewmodel/avatars";
import { approach, NEUTRAL, poseAt, type PoseMode } from "../viewmodel/pose";

/**
 * Karakter 3D di kanvas WebGL. three.js dimuat malas (hanya saat tab Regang dibuka). Bila WebGL tidak tersedia,
 * `fallback` ditampilkan. `calm` (kurangi animasi) menampilkan postur gerakan yang diam.
 */
export function Avatar3D({
  avatar,
  mode,
  calm,
  fallback,
}: {
  avatar: AvatarSpec;
  mode: PoseMode;
  calm: boolean;
  fallback: ReactNode;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const modeRef = useRef<PoseMode>(mode);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    modeRef.current = mode;
  }, [mode]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    let alive = true;
    let raf = 0;
    let teardown = () => {};

    void (async () => {
      try {
        const [THREE, { buildAvatar }] = await Promise.all([
          import("three"),
          import("./avatarRig"),
        ]);
        if (!alive) return;
        const renderer = new THREE.WebGLRenderer({ canvas, alpha: true, antialias: true });
        renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
        renderer.setSize(canvas.clientWidth, canvas.clientHeight, false);
        renderer.setClearColor(0x000000, 0);
        const camera = new THREE.PerspectiveCamera(
          28,
          canvas.clientWidth / canvas.clientHeight,
          0.1,
          50,
        );
        camera.position.set(0, 1.1, 5.5);
        camera.lookAt(0, 1.1, 0);
        const rig = buildAvatar(THREE, avatar);
        teardown = () => {
          rig.dispose();
          renderer.dispose();
        };

        let t = 0;
        let tMode = 0;
        let lastMode = modeRef.current;
        let pose = { ...NEUTRAL };
        let last = performance.now();
        const step = (now: number) => {
          const dt = Math.min((now - last) / 1000, 0.1);
          last = now;
          t += dt;
          if (modeRef.current !== lastMode) {
            lastMode = modeRef.current;
            tMode = t;
          }
          // Mode "kurangi animasi": postur gerakan dibekukan di titik yang jelas, tanpa gerak.
          pose = calm ? poseAt(lastMode, 1) : approach(pose, poseAt(lastMode, t - tMode), dt);
          rig.apply(pose, calm ? 0.5 : t);
          renderer.render(rig.scene, camera);
          raf = requestAnimationFrame(step);
        };
        raf = requestAnimationFrame(step);
      } catch {
        if (alive) setFailed(true);
      }
    })();

    return () => {
      alive = false;
      cancelAnimationFrame(raf);
      teardown();
    };
  }, [avatar, calm]);

  if (failed) return <>{fallback}</>;
  return <canvas ref={canvasRef} className="avatar-3d" aria-hidden="true" />;
}
