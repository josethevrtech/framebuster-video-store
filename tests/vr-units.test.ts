import { test } from 'node:test';
import assert from 'node:assert/strict';
import { Group, PerspectiveCamera, Vector3 } from 'three';
import { XR_METRES_TO_STORE_FEET } from '../src/vr-units.ts';

test('tracked head and controller positions retain physical size in the store after locomotion', () => {
  const rig = new Group();
  rig.scale.setScalar(XR_METRES_TO_STORE_FEET);
  rig.position.set(20, 0, -10);
  const head = new PerspectiveCamera();
  head.position.set(0, 1.8, 0);
  const controller = new Group();
  controller.position.set(0.3, 1.2, -0.4);
  rig.add(head, controller);
  rig.updateMatrixWorld(true);
  const h = head.getWorldPosition(new Vector3());
  const c = controller.getWorldPosition(new Vector3());
  assert.ok(Math.abs(h.y * 0.3048 - 1.8) < 1e-10);
  assert.ok(Math.abs((c.x - 20) * 0.3048 - 0.3) < 1e-10);
  assert.ok(Math.abs((c.z + 10) * 0.3048 + 0.4) < 1e-10);
  // A physical 64mm IPD needs a corresponding 0.21ft eye separation.
  assert.ok(Math.abs(0.064 * rig.scale.x * 0.3048 - 0.064) < 1e-10);
  const trackingRoot = new Group();
  trackingRoot.scale.setScalar(1 / XR_METRES_TO_STORE_FEET);
  const heldCase = new Group();
  heldCase.position.set(0, 0, -1);
  trackingRoot.add(heldCase);
  head.add(trackingRoot);
  rig.updateMatrixWorld(true);
  assert.ok(Math.abs(heldCase.getWorldPosition(new Vector3()).z - (h.z - 1)) < 1e-10);
  assert.ok(Math.abs(heldCase.getWorldScale(new Vector3()).x - 1) < 1e-10);
});
