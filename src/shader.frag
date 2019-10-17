in vec2 v_uv;

out vec4 frag;

uniform float time;
uniform float trigger_time;
uniform vec2 resolution;

void main() {
    vec2 pos = v_uv - 0.5;
    pos.x *= resolution.x / resolution.y;

    float a = atan(pos.y, pos.x);
    float d = length(pos);

    a += .1 * cos(time * 100. + d * 20.) * exp(trigger_time - time + d) ;
    d += .03 * cos(time * 40. + a * 6.28 * 10.) * exp(trigger_time - time + d) ;

    pos.x = d * cos(a);
    pos.y = d * sin(a);

    vec2 grid = pos * 5.;
    grid.x += sin(time * .1) * 5.;
    grid.y += sin(time * .08) * 4.8;
    grid = mod(grid + .5, 1.) - .5;

    float c = smoothstep(0.02, 0.01, abs(grid.x)) + smoothstep(0.02, 0.01, abs(grid.y));

    frag = vec4(vec3(c), 1.);
}
