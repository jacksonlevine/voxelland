#version 330 core
in vec3 vertexColor;
in vec2 TexCoord;
in vec3 pos;
in vec2 TexBase;
out vec4 FragColor;
uniform sampler2D ourTexture;
uniform vec3 camPos;
uniform float viewDistance;
uniform float ambientBrightMult;
uniform float underWater;
uniform vec3 camDir;

uniform float sunset;
uniform float sunrise;

float similarity(vec3 dir1, vec3 dir2) {
    return (dot(normalize(dir1), normalize(dir2)) + 1.0) * 0.5;
}
void main()
{

    // Calculate the horizontal and vertical distances from the corner
    float dx = abs(TexCoord.x - TexBase.x);
    float dy = abs(TexCoord.y - TexBase.y);

    // Check if the fragment is within the bounds of the quad
    if (dx > 0.02941176470588235294117647058824 || dy > 0.02941176470588235294117647058824) {
        discard; // Discard the fragment if its outside the quad
    }

    vec4 texColor = texture(ourTexture, TexCoord);
    FragColor = texColor * vec4(vertexColor, 1.0);

    vec3 west = vec3(0.0,0.0,-1.0);
    vec3 east = vec3(0.0,0.0,1.0);

    vec4 fogColor = vec4(0.7, 0.8, 1.0, 1.0) * vec4(ambientBrightMult, ambientBrightMult, ambientBrightMult, 1.0);

    fogColor = mix(fogColor, vec4(1.0, 0.651, 0.0, 1.0), (similarity(camDir, east) * 0.7) * sunrise);
    fogColor = mix(fogColor, vec4(1.0, 0.651, 0.0, 1.0), (similarity(camDir, west) * 0.7) * sunset); 

    float distance = (distance(pos, camPos)/(viewDistance*5.0f))/5.0f;

    if(underWater == 1.0) {
        fogColor = vec4(0.0, 0.0, 0.6, 1.0) * vec4(ambientBrightMult, ambientBrightMult, ambientBrightMult, 1.0);
        distance = distance * 10.0;
    }

    

    

    if(FragColor.a < 0.4) {
        discard;
    }

    if(FragColor.a < 1.0) {
        FragColor.a += distance*2.5f;
    }

    FragColor = mix(FragColor, fogColor, min(1, max(distance, 0)));
}