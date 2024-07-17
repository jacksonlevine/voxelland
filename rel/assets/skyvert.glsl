#version 450 core
out vec2 v_uv;
uniform float cpitch;
void main()
{
    uint idx = gl_VertexID;
    gl_Position = vec4((idx >> 1), idx & 1, 0.0, 0.5) * 4.0 - 1.0;
    v_uv = vec2(gl_Position.xy + 1.0 + (cpitch / 62));
}
