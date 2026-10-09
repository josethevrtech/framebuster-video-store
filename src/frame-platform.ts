export function isFrameLibrary(): boolean {
  return typeof location !== 'undefined'
    && new URLSearchParams(location.search).get('frame') === '1';
}
