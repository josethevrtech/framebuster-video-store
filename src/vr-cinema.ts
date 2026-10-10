import * as THREE from 'three';
import { reportFrame } from './frame-diagnostics';
import { controllerVisual, disposeControllerVisual } from './vr-controller-visual';

export interface CinemaControls {
  togglePlayPause(): void;
  seekBy(seconds: number): void;
}

/** A metre-scale theatre sharing the existing player's decoder and watch history. */
export class VRCinema {
  private pending = false;
  private generation = 0;
  private session: XRSession | null = null;
  private renderer: THREE.WebGLRenderer | null = null;
  private cleanup: (() => void) | null = null;

  async enter(video: HTMLVideoElement, controls: CinemaControls): Promise<void> {
    if (this.pending || this.session || !navigator.xr) return;
    this.pending = true;
    const generation = ++this.generation;
    let session: XRSession | null = null;
    try {
      // Called directly from a button gesture; do not await module loading first.
      session = await navigator.xr.requestSession('immersive-vr', { optionalFeatures: ['local-floor'] });
      if (generation !== this.generation) { await session.end(); return; }
      this.session = session;
      reportFrame(session, performance.now(), false, 'cinema', video);
      session.addEventListener('end', () => { if (this.session === session) this.release(); }, { once: true });
      const canvas = document.createElement('canvas');
      const context = canvas.getContext('webgl2', { xrCompatible: true, antialias: false, alpha: false });
      if (!context) throw new Error('WebGL2 unavailable');
      const renderer = new THREE.WebGLRenderer({ canvas, context, antialias: false, alpha: false });
      this.renderer = renderer;
      renderer.xr.enabled = true;
      renderer.xr.setReferenceSpaceType(session.enabledFeatures?.includes('local-floor') ? 'local-floor' : 'local');
      renderer.xr.setFramebufferScaleFactor(0.8);
      renderer.domElement.style.cssText = 'position:fixed;inset:0;width:100%;height:100%;z-index:100000';
      document.body.append(renderer.domElement);
      const scene = new THREE.Scene();
      scene.background = new THREE.Color('#090b13');
      const camera = new THREE.PerspectiveCamera(70, 1, 0.05, 40);
      const theatre = new THREE.Group();
      scene.add(theatre);
      const geometries: THREE.BufferGeometry[] = [];
      const materials: THREE.Material[] = [];
      const textures: THREE.Texture[] = [];
      const plane = (width: number, height: number, material: THREE.Material) => {
        const geometry = new THREE.PlaneGeometry(width, height);
        geometries.push(geometry); materials.push(material);
        const mesh = new THREE.Mesh(geometry, material);
        theatre.add(mesh);
        return mesh;
      };
      const texture = new THREE.VideoTexture(video);
      texture.colorSpace = THREE.SRGBColorSpace;
      textures.push(texture);
      const screen = plane(6, 1, new THREE.MeshBasicMaterial({ map: texture, toneMapped: false }));
      screen.position.set(0, 1.8, -4.2);
      const border = plane(6.16, 1, new THREE.MeshBasicMaterial({ color: '#242637' }));
      border.position.set(0, 1.8, -4.23);
      const floor = plane(24, 24, new THREE.MeshBasicMaterial({ color: '#121521', side: THREE.DoubleSide }));
      floor.rotation.x = -Math.PI / 2;
      const captionCanvas = document.createElement('canvas');
      captionCanvas.width = 1536; captionCanvas.height = 256;
      const ctx = captionCanvas.getContext('2d')!;
      const captions = new THREE.CanvasTexture(captionCanvas);
      captions.colorSpace = THREE.SRGBColorSpace;
      textures.push(captions);
      const captionPlane = plane(5.6, 0.93, new THREE.MeshBasicMaterial({ map: captions, transparent: true, depthWrite: false }));
      captionPlane.position.set(0, 0.7, -4.15);
      const instructionCanvas = document.createElement('canvas');
      instructionCanvas.width = 1536; instructionCanvas.height = 128;
      const instructionCtx = instructionCanvas.getContext('2d')!;
      instructionCtx.fillStyle = '#c0c7dc'; instructionCtx.font = '32px sans-serif';
      instructionCtx.textAlign = 'center';
      instructionCtx.fillText('Trigger: play / pause     Right stick: skip 10 seconds     Grip: return', 768, 58);
      const instructions = new THREE.CanvasTexture(instructionCanvas);
      instructions.colorSpace = THREE.SRGBColorSpace; textures.push(instructions);
      plane(5.6, 0.47, new THREE.MeshBasicMaterial({ map: instructions, transparent: true })).position.set(0, 0.25, -4.1);
      const end = () => { void session?.end().catch(() => this.release()); };
      const controllers: THREE.Group[] = [];
      for (let i = 0; i < 2; i++) {
        const controller = renderer.xr.getController(i);
        const visual = controllerVisual(i);
        controller.add(visual); controllers.push(visual);
        controller.addEventListener('selectstart', () => controls.togglePlayPause());
        controller.addEventListener('squeezestart', end);
        scene.add(controller);
      }
      this.cleanup = () => {
        controllers.forEach(disposeControllerVisual);
        for (const geometry of geometries) geometry.dispose();
        for (const material of materials) material.dispose();
        for (const item of textures) item.dispose();
      };
      await renderer.xr.setSession(session);
      if (generation !== this.generation || this.session !== session) { await session.end(); return; }
      let aligned = false, aspect = 0, lastCaption = '', nextCaptionAt = 0, seekReady = true;
      const euler = new THREE.Euler(0, 0, 0, 'YXZ');
      const headPosition = new THREE.Vector3();
      const headRotation = new THREE.Quaternion();
      renderer.setAnimationLoop((time, frame) => {
        if (!frame) return;
        const pose = frame.getViewerPose(renderer.xr.getReferenceSpace()!);
        if (!aligned && pose) {
          headPosition.set(pose.transform.position.x, pose.transform.position.y, pose.transform.position.z);
          headRotation.set(pose.transform.orientation.x, pose.transform.orientation.y, pose.transform.orientation.z, pose.transform.orientation.w);
          euler.setFromQuaternion(headRotation);
          theatre.position.set(headPosition.x, headPosition.y - 1.6, headPosition.z);
          theatre.rotation.y = euler.y;
          aligned = true;
        }
        const nextAspect = video.videoWidth && video.videoHeight ? video.videoWidth / video.videoHeight : 16 / 9;
        if (aspect !== nextAspect) {
          aspect = nextAspect;
          screen.scale.y = 6 / aspect;
          border.scale.y = 6 / aspect + 0.16;
        }
        // WebVTT lives outside the video pixels; draw active cues separately in XR.
        if (time >= nextCaptionAt) {
          nextCaptionAt = time + 100;
          let text = '';
          for (let i = 0; i < video.textTracks.length; i++) {
            const track = video.textTracks[i];
            if (track.mode !== 'showing') continue;
            for (let j = 0; j < (track.activeCues?.length ?? 0); j++) {
              const cue = track.activeCues![j];
              if (cue instanceof VTTCue) text += `${cue.text.replace(/<[^>]*>/g, '')}\n`;
            }
          }
          if (text !== lastCaption) {
            lastCaption = text; ctx.clearRect(0, 0, 1536, 256);
            ctx.font = '48px sans-serif'; ctx.textAlign = 'center';
            ctx.lineWidth = 8; ctx.strokeStyle = '#000'; ctx.fillStyle = '#fff';
            const lines = text.trim().split('\n').slice(0, 3);
            lines.forEach((line, i) => { ctx.strokeText(line, 768, 64 + i * 64, 1450); ctx.fillText(line, 768, 64 + i * 64, 1450); });
            captions.needsUpdate = true;
          }
        }
        for (const source of frame.session.inputSources) {
          if (source.handedness !== 'right' || !source.gamepad) continue;
          const axes = source.gamepad.axes;
          const x = axes.length >= 4 ? axes[2] : axes[0] ?? 0;
          if (Math.abs(x) < 0.3) seekReady = true;
          else if (Math.abs(x) > 0.7 && seekReady) { seekReady = false; controls.seekBy(x > 0 ? 10 : -10); }
        }
        reportFrame(frame.session, time, !!pose, 'cinema', video);
        renderer.render(scene, camera);
      });
    } catch (error) {
      if (session) await session.end().catch(() => {});
      this.release();
      throw error;
    } finally { this.pending = false; }
  }

  stop(): void {
    ++this.generation;
    const session = this.session;
    if (session) void session.end().catch(() => {});
    this.release();
  }

  private release(): void {
    if (this.session) reportFrame(null, performance.now(), false, 'cinema');
    this.session = null;
    this.renderer?.setAnimationLoop(null);
    this.cleanup?.(); this.cleanup = null;
    this.renderer?.domElement.remove();
    this.renderer?.dispose(); this.renderer?.forceContextLoss(); this.renderer = null;
  }
}
