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
        vec3 tangent=(dp1*uv2.y-dp2*uv1.y)/determinant;
        vec3 rawBitangent=(-dp1*uv2.x+dp2*uv1.x)/determinant;
        tangent=normalize(tangent-n*dot(n,tangent));
        vec3 bitangent=normalize(cross(n,tangent))*sign(dot(cross(n,tangent),rawBitangent));
        vec3 mapped=sampleMap(2.)*2.-1.;
        n=normalize(mat3(tangent,bitangent,n)*mapped);
    }
    vec3 light=normalize(vec3(-.18,.96,.20)),view=normalize(eyeDirection);
    vec3 halfVector=normalize(light+view);
    float nl=max(dot(n,light),0.),nv=max(dot(n,view),.001);
    float nh=max(dot(n,halfVector),0.),vh=max(dot(view,halfVector),0.);
    float rough=clamp(arm.g,.18,1.),metal=clamp(arm.b,0.,1.);
    vec3 f0=mix(vec3(.04),albedo,metal);
    vec3 fresnel=f0+(1.-f0)*pow(1.-vh,5.);
    float alpha=rough*rough,a2=alpha*alpha;
    float denominator=nh*nh*(a2-1.)+1.;
    float distribution=a2/(3.14159265*denominator*denominator);
    float k=(rough+1.)*(rough+1.)*.125;
    float geometry=(nv/(nv*(1.-k)+k))*(nl/(nl*(1.-k)+k));
    vec3 specular=distribution*geometry*fresnel/max(4.*nv*nl,.001);
    vec3 diffuse=(1.-fresnel)*(1.-metal)*albedo/3.14159265;
    vec3 ambient=mix(vec3(.17,.15,.13),vec3(.35,.37,.40),n.y*.5+.5);
    float ao=clamp(arm.r,.12,1.);
    vec3 color=albedo*ambient*ao*(1.-metal*.65)
        +(diffuse+specular)*vec3(2.0,1.92,1.80)*nl;
    vec3 reflected=reflect(-view,n);
    float ceiling=pow(max(reflected.y,0.),mix(48.,3.,rough));
    color+=f0*ceiling*.25*(1.-rough)*ao;
    outputColor=vec4(color/(vec3(1.)+color*.12),1.);
}
