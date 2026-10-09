#version 450
layout(location = 0) in vec3 litColor;
layout(location = 0) out vec4 outputColor;
void main() {
    outputColor = vec4(litColor, 1.0);
}
