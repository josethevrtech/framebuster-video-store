import * as THREE from 'three';

/** Offline controller markers and aiming rays, authored in tracking metres. */
export function controllerVisual(index: number): THREE.Group {
  const visual = new THREE.Group();
  const color = index === 0 ? 0x70cfff : 0xffd582;
  const handle = new THREE.Mesh(new THREE.BoxGeometry(0.035, 0.06, 0.09), new THREE.MeshBasicMaterial({ color }));
  handle.position.set(0, -0.02, 0.02);
  visual.add(handle);
  const ray = new THREE.Line(new THREE.BufferGeometry().setFromPoints([
    new THREE.Vector3(0, 0, -0.04), new THREE.Vector3(0, 0, -3),
  ]), new THREE.LineBasicMaterial({ color, transparent: true, opacity: 0.5 }));
  visual.add(ray);
  return visual;
}

export function disposeControllerVisual(visual: THREE.Group): void {
  visual.traverse(object => {
    if (object instanceof THREE.Mesh || object instanceof THREE.Line) {
      object.geometry.dispose();
      if (Array.isArray(object.material)) object.material.forEach(material => material.dispose());
      else object.material.dispose();
    }
  });
  visual.removeFromParent();
}
