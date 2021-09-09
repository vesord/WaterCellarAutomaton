#version 460 core

in vec4 passColor;

out vec4 Color;

void main() {
    Color = passColor;
}