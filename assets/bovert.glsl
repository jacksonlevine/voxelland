#version 450 core
layout (location = 0) in vec3 position;
layout (location = 1) in float cornerID;

out vec2 TexCoord;

uniform mat4 mvp;


uniform vec3 blockPosition;
uniform float breakPhase;

uniform float walkbob;

void main()
{
    vec3 bob = vec3(0.0, ((sin(walkbob) )/20.0), 0.0) + vec3(0.0, 0.5, 0.0);
    gl_Position = mvp * vec4(((position - bob) + blockPosition + vec3(0.5, 0.5, 0.5)) , 1.0);

    vec2 baseuv = vec2(0, 15);

        //Texture stuff
    float onePixel = 0.00183823529411764705882352941176f;     //  1/544      Padding
    float textureWidth = 0.02941176470588235294117647058824f; // 16/544      16 pixel texture width
    float texSlotWidth = 0.03308823529411764705882352941176f;

       vec2 texOffsets[6] = {
        vec2(onePixel, -onePixel),
        vec2(onePixel + textureWidth, -onePixel),
        vec2(onePixel + textureWidth, -(onePixel + textureWidth)),
        vec2(onePixel + textureWidth, -(onePixel + textureWidth)),
        vec2(onePixel, -(onePixel + textureWidth)),
        vec2(onePixel, -onePixel)
    };

    float u = baseuv.x; 
    float v = baseuv.y;       

    vec2 uvOffset = texOffsets[int(cornerID)];
    vec2 uv = vec2((u * texSlotWidth) + uvOffset.x, (1.0f - (v * texSlotWidth)) + uvOffset.y);


    TexCoord = uv + vec2(breakPhase * texSlotWidth, 0);
}