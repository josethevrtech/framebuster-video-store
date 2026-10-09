#version 450
layout(binding = 0) uniform sampler2D artwork;
layout(location = 0) in vec2 textureUV;
layout(location = 0) out vec4 outputColor;
void main() {
    vec4 pixel = texture(artwork, textureUV);
    if (pixel.a < 0.5) discard;
    outputColor = vec4(pixel.rgb, 1.0);
}
