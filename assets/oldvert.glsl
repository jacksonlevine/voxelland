#version 330 core
layout (location = 0) in vec3 position;
layout (location = 1) in float blockBright;
layout (location = 2) in float ambientBright;
layout (location = 3) in vec2 uv;
layout (location = 4) in vec2 uvbase;
out vec3 vertexColor;
out vec2 TexCoord;

out vec3 pos;
uniform mat4 mvp;
uniform vec3 camPos;
uniform float ambientBrightMult;
uniform float viewDistance;
void main()
{
    

    float ambBright = ambientBrightMult * ambientBright;

    float distance = pow(distance(position, camPos)/(5), 2)/5.0f;
    gl_Position = mvp * vec4(position , 1.0);

    float bright = min(16.0f, blockBright + ambBright);

    

    vertexColor = vec3(bright/16.0f, bright/16.0f, bright/16.0f);
    TexCoord = uv;
    
    pos = position;
}