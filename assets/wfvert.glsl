#version 330 core
layout (location = 0) in vec3 position;
uniform mat4 mvp;
uniform vec3 translation;

uniform float walkbob;

void main()
{

    vec3 bob = vec3(0.0, ((sin(walkbob) )/20.0), 0.0) + vec3(0.0, 0.5, 0.0);

    gl_Position = mvp * vec4(((position - bob) + translation + vec3(0.5, 0.5, 0.5)), 1.0);

}