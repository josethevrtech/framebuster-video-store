#version 450
layout(binding=0) uniform sampler2D materials;
layout(location=0) in vec3 surfaceNormal;
layout(location=1) in vec3 eyeDirection;
layout(location=2) in vec2 textureUV;
layout(location=3) in vec3 surfacePosition;
layout(location=0) out vec4 outputColor;
vec3 sampleMap(float index) {
    vec2 uv=clamp(textureUV,vec2(.0005),vec2(.9995));
    return texture(materials,vec2((uv.x+index)/3.,uv.y)).rgb;
}
void main() {
    vec3 albedo=pow(sampleMap(0.),vec3(2.2));
    vec3 arm=sampleMap(1.);
    vec3 n=normalize(surfaceNormal);
    vec3 dp1=dFdx(surfacePosition),dp2=dFdy(surfacePosition);
    vec2 uv1=dFdx(textureUV),uv2=dFdy(textureUV);
    float determinant=uv1.x*uv2.y-uv1.y*uv2.x;
    if(abs(determinant)>.0000001) {
        vec3 tangent=normalize((dp1*uv2.y-dp2*uv1.y)/determinant);
        vec3 bitangent=normalize((-dp1*uv2.x+dp2*uv1.x)/determinant);
        vec3 mapped=sampleMap(2.)*2.-1.;
        n=normalize(mat3(tangent,bitangent,n)*mapped);
    }
    vec3 light=normalize(vec3(-.3,.8,.5)),view=normalize(eyeDirection);
    float diffuse=max(dot(n,light),0.),rough=clamp(arm.g,.12,1.),metal=arm.b;
    vec3 f0=mix(vec3(.04),albedo,metal);
    float highlight=pow(max(dot(n,normalize(light+view)),0.),mix(128.,4.,rough));
    vec3 fresnel=f0+(1.-f0)*pow(1.-max(dot(n,view),0.),5.);
    vec3 color=albedo*(.32+.58*diffuse)*mix(.55,1.,arm.r)*(1.-metal*.6)
        +fresnel*(highlight*.65+.10*(1.-rough));
    outputColor=vec4(color,1.);
}
