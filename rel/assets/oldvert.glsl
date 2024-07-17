#version 330 core
layout (location = 0) in vec3 position;
layout (location = 1) in uint blockRgb;
layout (location = 2) in float ambientBright;
layout (location = 3) in vec2 uv;
layout (location = 4) in vec2 uvbase;
out vec3 vertexColor;
out vec2 TexCoord;
out vec3 blockColor;

out vec3 pos;
uniform mat4 mvp;
uniform vec3 camPos;
uniform float ambientBrightMult;
uniform float viewDistance;

uniform float walkbob;
void main()
{

    // Decode the RGB value from the 32-bit attribute (4 bits per color channel)
    uint r = (blockRgb & uint(0x00000F00)) >> 8;
    uint g = (blockRgb & uint(0x000000F0)) >> 4;
    uint b = (blockRgb & uint(0x0000000F));

    float ambBright = ambientBrightMult * ambientBright;

    float distance = pow(distance(position, camPos)/(5), 2)/5.0f;

    vec3 bob = vec3(0.0, ((sin(walkbob) )/20.0), 0.0) + vec3(0.0, 0.3, 0.0);

    gl_Position = mvp * vec4(position - bob , 1.0);

    float bright = min(16.0f, ambBright);

    blockColor = vec3(float(r)/16.0f, float(g)/16.0f, float(b)/16.0f);

    

    vertexColor = vec3(bright/16.0f, bright/16.0f, bright/16.0f) + blockColor;
    vertexColor = vec3(min(1.0, vertexColor.r), min(1.0, vertexColor.g), min(1.0, vertexColor.b));
    TexCoord = uv;
    
    pos = position;

    
}