#version 450
layout(location=0) in vec4 position;
layout(location=1) in vec4 normal;
layout(location=2) in vec4 uv;
layout(location=0) out vec3 surfaceNormal;
layout(location=1) out vec3 eyeDirection;
layout(location=2) out vec2 textureUV;
layout(location=3) out vec3 surfacePosition;
layout(push_constant) uniform Parameters {
    vec4 eyeRotation; vec4 eyePosition; vec4 roomRotation; vec4 roomPosition; vec4 fov;
} p;
vec3 rotate(vec4 q,vec3 v) {
    return v+2.0*cross(q.xyz,cross(q.xyz,v)+q.w*v);
}
void main() {
    vec3 world=rotate(p.roomRotation,position.xyz)+p.roomPosition.xyz;
    vec3 eye=rotate(vec4(-p.eyeRotation.xyz,p.eyeRotation.w),world-p.eyePosition.xyz);
    float nearPlane=.02,farPlane=100.;
    gl_Position=vec4((2.*eye.x+(p.fov.y+p.fov.x)*eye.z)/(p.fov.y-p.fov.x),
        (2.*eye.y+(p.fov.w+p.fov.z)*eye.z)/(p.fov.w-p.fov.z),
        (-eye.z*farPlane-nearPlane*farPlane)/(farPlane-nearPlane),-eye.z);
    surfaceNormal=normalize(normal.xyz);
    surfacePosition=position.xyz;
    eyeDirection=rotate(vec4(-p.roomRotation.xyz,p.roomRotation.w),p.eyePosition.xyz-world);
    textureUV=uv.xy;
}
