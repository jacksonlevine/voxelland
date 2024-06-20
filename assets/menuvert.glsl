#version 330 core
layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 texcoord;
layout (location = 2) in float elementid;

out vec2 TexCoord;
out float elementID;

uniform vec2 translation;

void main()
{
    if(elementid == 222.0) {
        gl_Position = vec4(pos + translation, 0.0, 1.0);
    } else {
        gl_Position = vec4(pos, 0.0, 1.0);
    }
    

    TexCoord = texcoord;
    elementID = elementid;
}