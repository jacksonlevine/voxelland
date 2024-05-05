#version 330 core
layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 texcoord;
layout (location = 2) in float elementid;

out vec2 TexCoord;
out float elementID;

void main()
{
    gl_Position = vec4(pos, 0.0, 1.0);

    TexCoord = texcoord;
    elementID = elementid;
}