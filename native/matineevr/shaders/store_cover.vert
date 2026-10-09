#version 450
layout(location = 0) in vec4 position;
layout(location = 1) in vec4 uv;
layout(location = 0) out vec2 textureUV;
layout(push_constant) uniform Parameters {
    vec4 eyeRotation;
    vec4 eyePosition;
    vec4 roomRotation;
    vec4 roomPosition;
    vec4 fov;
} p;
vec3 rotate(vec4 q, vec3 v) {
    return v + 2.0 * cross(q.xyz, cross(q.xyz, v) + q.w * v);
}
void main() {
    vec3 world = rotate(p.roomRotation, position.xyz) + p.roomPosition.xyz;
    vec3 eye = rotate(vec4(-p.eyeRotation.xyz, p.eyeRotation.w), world - p.eyePosition.xyz);
    float nearPlane = 0.02;
    float farPlane = 100.0;
    gl_Position = vec4(
        (2.0 * eye.x + (p.fov.y + p.fov.x) * eye.z) / (p.fov.y - p.fov.x),
        (2.0 * eye.y + (p.fov.w + p.fov.z) * eye.z) / (p.fov.w - p.fov.z),
        (-eye.z * farPlane - nearPlane * farPlane) / (farPlane - nearPlane), -eye.z);
    textureUV = uv.xy;
}
