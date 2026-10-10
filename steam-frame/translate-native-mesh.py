from pathlib import Path
import struct
import sys

source,output=map(Path,sys.argv[1:3])
delta=list(map(float,sys.argv[3:]))
assert len(delta)==3
data=bytearray(source.read_bytes())
header=28 if data[:8]==b'FBCONS01' else 12
assert data[:8] in [b'FBCONS01',b'FBPROP01']
assert (len(data)-header)%48==0
for offset in range(header,len(data),48):
    point=struct.unpack_from('<3f',data,offset)
    struct.pack_into('<3f',data,offset,*[point[i]+delta[i] for i in range(3)])
output.write_bytes(data)
print('Repositioned native mesh; materials and normals preserved.')
