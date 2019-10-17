in vec2 pos;
in vec2 uv;

out vec2 v_uv;

void main() {
  gl_Position = vec4(pos, 0., 1.);
  v_uv = uv;
}
