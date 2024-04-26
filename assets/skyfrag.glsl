#version 450 core
uniform vec4 top_color;
uniform vec4 bot_color;
uniform float brightMult;
uniform float sunrise;
uniform float sunset;
uniform vec3 camDir;
in vec2 v_uv;
out vec4 frag_color;

float similarity(vec3 dir1, vec3 dir2) {
    return (dot(normalize(dir1), normalize(dir2)) + 1.0) * 0.5;
}

void main()
{
    vec3 east = vec3(0, 0, 1);
    vec3 west = vec3(0, 0, -1);
    vec4 botColor = mix(bot_color * vec4(brightMult, brightMult, brightMult, 1.0f), bot_color, (similarity(camDir, east)) * sunrise);
    botColor = mix(botColor, bot_color, (similarity(camDir, west)) * sunset);
    frag_color = mix(botColor, top_color * vec4(brightMult, brightMult, brightMult, 1.0f), max(min(pow(v_uv.y-0.4, 1.0), 1.0), 0.0));

}