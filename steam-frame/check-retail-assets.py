from pathlib import Path
import json
import math
import struct
import sys

mesh,inspection=map(Path,sys.argv[1:])
report=json.loads(inspection.read_text())
data=mesh.read_bytes()
assert data[:8]==b'FBPROP01'
count=struct.unpack_from('<I',data,8)[0]
assert len(data)==12+count*48
rows=[struct.unpack_from('<12f',data,12+i*48) for i in range(count)]
assert all(math.isfinite(x) for row in rows for x in row)
assert all(0<=row[8]<=1 and 0<=row[9]<=1 for row in rows)
counter_count=report['models']['counter']['triangles']*3
counter=rows[:counter_count]
top=-1.5+2.82*.3048
surfaces=[counter[i:i+3] for i in range(0,len(counter),3) if all(abs(p[1]-top)<.001 and p[5]>.99 for p in counter[i:i+3])]
def cross(a,b,p): return (b[0]-a[0])*(p[1]-a[1])-(b[1]-a[1])*(p[0]-a[0])
def supported(p):
    for triangle in surfaces:
        a,b,c=[(q[0],q[2]) for q in triangle]
        signs=[cross(a,b,p),cross(b,c,p),cross(c,a,p)]
        if min(signs)>=-.00001 or max(signs)<=.00001: return True
    return False
end=counter_count+report['models']['registers']['triangles']*3
feet={(r[0],r[2]) for r in rows[counter_count:end] if abs(r[1]-top)<.001}
unsupported=[p for p in feet if not supported(p)]
assert not unsupported,(len(unsupported),len(feet),unsupported[:6])
layout=report['layout']
assert layout['cover'][1]-.097-.012>=layout['top']-.001
assert layout['cover'][2]>=report['models']['music-cabinet']['bounds'][0][2]
print(f'Verified {len(feet)} register contact points; atlas coordinates, finite vertices and CD stand support are valid.')
