#version 330 core
out vec4 FragColor;
in vec2 TexCoord;
uniform sampler2D ourTexture;

uniform vec3 camPos;
uniform float viewDistance;
uniform vec4 fogCol;

uniform float sunset;
uniform float sunrise;
uniform vec3 camDir;

uniform vec3 pos;

uniform float ambientBrightMult;

float similarity(vec3 dir1, vec3 dir2) {
    return (dot(normalize(dir1), normalize(dir2)) + 1.0) * 0.5;
}

void main() {


    vec3 west = vec3(0.0,0.0,-1.0);
    vec3 east = vec3(0.0,0.0,1.0);

    vec4 fogColor = fogCol * vec4(ambientBrightMult, ambientBrightMult, ambientBrightMult, 1.0);

    fogColor = mix(fogColor, vec4(1.0, 0.651, 0.0, 1.0), (similarity(camDir, east) * 0.7) * sunrise);
    fogColor = mix(fogColor, vec4(1.0, 0.651, 0.0, 1.0), (similarity(camDir, west) * 0.7) * sunset); 



    float distance = (distance(pos, camPos)/(viewDistance*5.0f))/5.0f;
    FragColor = mix(FragColor, fogColor, min(1, max(distance, 0)));



    vec4 texColor = texture(ourTexture, TexCoord);
    FragColor = texColor  * vec4(ambientBrightMult, ambientBrightMult, ambientBrightMult, 1.0);
    FragColor = mix(FragColor, fogColor, min(1, max(distance, 0)));
}