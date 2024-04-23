#version 450 core
layout (location = 0) in uint u32;
layout (location = 1) in uint u8;
out vec3 vertexColor;
out vec2 TexCoord;
out vec3 pos;
uniform vec3 chunkpos;
uniform mat4 mvp;
uniform vec3 camPos;
uniform float ambientBrightMult;
uniform float viewDistance;
void main()
{
    float lx = float((u32 >> 28) & 0xF);       // Top 4 bits for x
    float ly = float((u32 >> 20) & 0xFF);      // Next 8 bits for y
    float lz = float((u32 >> 16) & 0xF);       // Next 4 bits for z

    vec3 position = vec3(lx, ly, lz) + (chunkpos * 15);

    uint cornerID = ((u32 >> 12) & 0xF);  // Next 4 bits for corner
    float ambientBright = float((u32 >> 8) & 0xF); // Next 4 bits for al
    float blockBright = float((u32 >> 4) & 0xF);   // Next 4 bits for bl

    //Texture stuff
    float onePixel = 0.00183823529411764705882352941176f;     //  1/544      Padding
    float textureWidth = 0.02941176470588235294117647058824f; // 16/544      16 pixel texture width

    vec2 texOffsets[6] = {
        vec2(onePixel, -onePixel),
        vec2(onePixel + textureWidth, -onePixel),
        vec2(onePixel + textureWidth, -(onePixel + textureWidth)),
        vec2(onePixel + textureWidth, -(onePixel + textureWidth)),
        vec2(onePixel, -(onePixel + textureWidth)),
        vec2(onePixel, -onePixel)
    };

    // Unpack from the u8 and cast to float
    float u = float((u8 >> 4) & 0xF);  // Top 4 bits for u
    float v = float(u8 & 0xF);         // Lower 4 bits for v

    vec2 uvOffset = texOffsets[cornerID];
    vec2 uv = vec2((u/16.0) + uvOffset.x, (v/16.0) + uvOffset.y);
    

    float ambBright = ambientBrightMult * ambientBright;

    float distance = pow(distance(position, camPos)/(5), 2)/5.0f;
    gl_Position = mvp * vec4(position , 1.0);

    float bright = min(16.0f, blockBright + ambBright);

    

    vertexColor = vec3(bright/16.0f, bright/16.0f, bright/16.0f);
    TexCoord = uv;
    pos = position;
}