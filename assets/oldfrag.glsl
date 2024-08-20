#version 330 core
in vec3 vertexColor;
in vec2 TexCoord;
in vec3 pos;

in float transparencychange;

out vec4 FragColor;
uniform sampler2D ourTexture;
uniform sampler2D weatherTexture;
uniform vec3 camPos;
uniform float viewDistance;
uniform float ambientBrightMult;
uniform float underWater;
uniform vec3 camDir;

uniform float sunset;
uniform float sunrise;

uniform float time;
uniform float weathertype;

uniform float renderingweather;

in vec3 blockColor;

float similarity(vec3 dir1, vec3 dir2) {
    return (dot(normalize(dir1), normalize(dir2)) + 1.0) * 0.5;
}
void main()
{   
    const float ruvh = 3.7647058823529411764705882352947;

    const float texw = 0.03308823529411764705882352941176;



    vec2 rainuvs[] = vec2[](
    vec2(0.498162, 0.998162),
    vec2(0.527574, 0.998162),
    vec2(0.527574, 0.998162  -  ruvh),
    vec2(0.527574, 0.998162  -  ruvh),
    vec2(0.498162, 0.998162  -  ruvh),
    vec2(0.498162, 0.998162));



    vec2 RealTexCoord = TexCoord;

    float rainx = 0.4981618;


    if (TexCoord.x >= rainx){

        if (weathertype == 1.0 ){
            RealTexCoord += vec2(texw * 3.0, 0.0); 

            RealTexCoord += vec2(0.0, time * -0.2  );
        } else if (weathertype == 2.0 ) {
            RealTexCoord += vec2(0.0, time * -0.8);
        } else {

        }

        
    }


    vec4 texColor = texture(ourTexture, RealTexCoord);
    if (TexCoord.x >= rainx){
        texColor = texture(weatherTexture, RealTexCoord);
    }
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

    FragColor.a += transparencychange;

}