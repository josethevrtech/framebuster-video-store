#version 450
layout(location = 0) in vec4 position;
layout(location = 1) in vec4 normal;
layout(location = 2) in vec4 color;
layout(location = 0) out vec3 litColor;
layout(push_constant) uniform Parameters {
    vec4 eyeRotation;
    vec4 eyePosition;
    vec4 handRotation;
    vec4 handPosition;
    vec4 fov;
} p;
vec3 rotate(vec4 q, vec3 v) {
    return v + 2.0 * cross(q.xyz, cross(q.xyz, v) + q.w * v);
}
void main() {
    vec3 world = rotate(p.handRotation, position.xyz) + p.handPosition.xyz;
    vec3 eye = rotate(vec4(-p.eyeRotation.xyz, p.eyeRotation.w), world - p.eyePosition.xyz);
    float nearPlane = 0.02;
    float farPlane = 100.0;
    gl_Position = vec4(
        (2.0 * eye.x + (p.fov.y + p.fov.x) * eye.z) / (p.fov.y - p.fov.x),
        (2.0 * eye.y + (p.fov.w + p.fov.z) * eye.z) / (p.fov.w - p.fov.z),
        (-eye.z * farPlane - nearPlane * farPlane) / (farPlane - nearPlane),
        -eye.z);
    vec3 n = normalize(rotate(p.handRotation, normal.xyz));
    float light = 0.55 + 0.45 * max(dot(n, normalize(vec3(-0.3, 0.8, 0.5))), 0.0);
    litColor = color.rgb * light;
}
