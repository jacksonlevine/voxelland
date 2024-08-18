#version 450 core
in vec3 vertexColor;
in vec2 TexCoord;
in vec3 pos;
in vec3 grassColor;

out vec4 FragColor;
uniform sampler2D ourTexture;
uniform vec3 camPos;
uniform float viewDistance;
uniform float ambientBrightMult;
uniform float underWater;
uniform vec3 camDir;

uniform vec4 fogCol;

uniform float sunset;
uniform float sunrise;
uniform float planet_y;
float similarity(vec3 dir1, vec3 dir2) {
    return (dot(normalize(dir1), normalize(dir2)) + 1.0) * 0.5;
}
void main()
{
    //vec4(0.0, 0.0, 0.6, 1.0)


    vec4 texColor = texture(ourTexture, TexCoord) + vec4(grassColor, 0.0);
    texColor = min(texColor, vec4(1.0));
    FragColor = texColor * vec4(vertexColor, 1.0);

    vec3 west = vec3(0.0,0.0,-1.0);
    vec3 east = vec3(0.0,0.0,1.0);

    vec4 fogColor = fogCol * vec4(ambientBrightMult, ambientBrightMult, ambientBrightMult, 1.0);

    fogColor = mix(fogColor, vec4(1.0, 0.651, 0.0, 1.0), (similarity(camDir, east) * 0.7) * sunrise);
    fogColor = mix(fogColor, vec4(1.0, 0.651, 0.0, 1.0), (similarity(camDir, west) * 0.7) * sunset); 

    float distance = (distance(pos, camPos)/(viewDistance*5.0f))/5.0f;

    
    if(underWater == 1.0) {
        fogColor = vec4(0.0, 0.0, 0.6, 1.0) * vec4(ambientBrightMult, ambientBrightMult, ambientBrightMult, 1.0);
        distance = distance * 10.0;
    }

    
    float space = abs(min(planet_y + 128, 0));
    

    if(FragColor.a < 0.4) {
        discard;
    }

    //Fresnel effect on semi-transparent stuff right here, I was wondering wtf this was I just remembered though
    if(FragColor.a < 1.0) {
        FragColor.a += distance*2.5f;
    }

    float spacedist = 300.0;

    FragColor = mix(FragColor, fogColor, min(1, max(distance, 0)));
    FragColor = FragColor - vec4(space/spacedist, space/spacedist, space/spacedist, 0.0);
}