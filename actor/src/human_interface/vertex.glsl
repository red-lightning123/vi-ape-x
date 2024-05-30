#version 330 core
layout (location = 0) in vec2 vpos;
out vec2 texCoord;
uniform vec2 center;
uniform vec2 dims;
void main() {
    // we vertically invert the texture coordinates because
    // OpenGL textures use a different coordinate system
    // to X11
    texCoord = (vec2(1.0f) + vec2(vpos.x, -vpos.y)) / 2.0f;
    gl_Position = vec4((center + dims * (vpos / 2.0f)) * vec2(1080.0f / 1920.0f, 1.0f), 0.0f, 1.0f);
}
