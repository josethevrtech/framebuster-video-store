import { test } from 'node:test';
import assert from 'node:assert/strict';
import { ownedMusicInput, channelVolumes } from './store-audio-route.mjs';

test('proximity changes target only the owned music process', () => {
  const music = { index: 7, properties: { 'application.process.id': '123', 'application.name': 'ffplay' } };
  const cinema = { index: 8, properties: { 'application.process.id': '124', 'application.name': 'FrameBuster' } };
  const otherMusic = { index: 9, properties: { 'application.process.id': '125', 'application.name': 'ffplay' } };
  assert.equal(ownedMusicInput([cinema, otherMusic, music], 123), music);
  assert.equal(ownedMusicInput([cinema, otherMusic], 123), undefined);
});

test('stereo gains remain bounded and mono is handled explicitly', () => {
  assert.deepEqual(channelVolumes([0.8, 0.4], 2), ['80.00%', '40.00%']);
  assert.deepEqual(channelVolumes([0.8, 0.4], 1), ['60.00%']);
  for (const gains of [[2, 1], [-1, 0], [NaN, 0], [], [0]])
    assert.throws(() => channelVolumes(gains, 2));
  assert.throws(() => channelVolumes([1, 1], 6));
});
