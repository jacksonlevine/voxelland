#version 330 core


out vec4 FragColor;
in vec2 TexCoord;
uniform sampler2D ourTexture;

uniform float viewDistance;
uniform vec4 fogCol;

uniform float sunset;
uniform float sunrise;
uniform vec3 camDir;


uniform float opacity;

uniform float ambientBrightMult;

uniform float time;

float similarity(vec3 dir1, vec3 dir2) {
    return (dot(normalize(dir1), normalize(dir2)) + 1.0) * 0.5;
}

float rand(vec2 c){
	return fract(sin(dot(c.xy ,vec2(12.9898,78.233))) * 43758.5453);
}

float noise(vec2 p, float freq ){
    float PI = 3.14159265358979323846;
	float unit = 0.05;
	vec2 ij = floor(p/unit);
	vec2 xy = mod(p,unit)/unit;
	//xy = 3.*xy*xy-2.*xy*xy*xy;
	xy = .5*(1.-cos(PI*xy));
	float a = rand((ij+vec2(0.,0.)));
	float b = rand((ij+vec2(1.,0.)));
	float c = rand((ij+vec2(0.,1.)));
	float d = rand((ij+vec2(1.,1.)));
	float x1 = mix(a, b, xy.x);
	float x2 = mix(c, d, xy.x);
	return mix(x1, x2, xy.y);
}

float pNoise(vec2 p, int res){
	float persistance = .5;
	float n = 0.;
	float normK = 0.;
	float f = 4.;
	float amp = 1.;
	int iCount = 0;
	for (int i = 0; i<50; i++){
		n+=amp*noise(p, f);
		f*=2.;
		normK+=amp;
		amp*=persistance;
		if (iCount == res) break;
		iCount++;
	}
	float nf = n/normK;
	return nf*nf*nf*nf;
}

void main() {


    vec3 west = vec3(0.0,0.0,-1.0);
    vec3 east = vec3(0.0,0.0,1.0);

    vec4 fogColor = fogCol * vec4(ambientBrightMult, ambientBrightMult, ambientBrightMult, 1.0);

    fogColor = mix(fogColor, vec4(1.0, 0.651, 0.0, 1.0), (similarity(camDir, east) * 0.7) * sunrise);
    fogColor = mix(fogColor, vec4(1.0, 0.651, 0.0, 1.0), (similarity(camDir, west) * 0.7) * sunset); 



 



    FragColor = vec4(FragColor.xyz, FragColor.w*opacity);

    float pn = pNoise(TexCoord + (vec2(0.0005, 0.0005) * time), 10) * 25.0;
    FragColor = vec4(ambientBrightMult, ambientBrightMult, ambientBrightMult, pn * 0.3);


}