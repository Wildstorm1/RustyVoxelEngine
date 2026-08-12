/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

#version 450

#pragma shader_stage(fragment)

layout(location = 0) out vec4 outColor;

layout(location = 0) in vec2 fragTexCoord;
layout(location = 1) in vec3 normal;
layout(location = 2) in float ambientOcclusion;

layout(binding = 2) uniform sampler2D texSampler;

void main() {
    float directionalLighting = (1.0 - (0.25 * max(dot(normal, -normalize(vec3(-1.0, -1.0, -1.0))), 0.0)));
    vec4 texture = texture(texSampler, fragTexCoord);
    outColor = vec4(ambientOcclusion * directionalLighting * texture.xyz, texture.w);
}